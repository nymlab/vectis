#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CanonicalAddr, ContractResult, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
use sc_wallet::{pub_key_to_address, query_verify_cosmos, RelayTransaction, WalletInfo};
use schemars::JsonSchema;
use std::collections::BTreeSet;
use std::fmt;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    User, CODE_ID, FACTORY, FROZEN, GUARDIANS, MULTISIG_ADDRESS, MULTISIG_CODE_ID, RELAYERS, USER,
};
use cw3_fixed_multisig::msg::{
    InstantiateMsg as FixedMultisigInstantiateMsg, QueryMsg as FixedMultisigQueryMsg, Voter,
};
use cw_storage_plus::Map;
use cw_utils::{Duration, Threshold, ThresholdResponse};
use sc_wallet::RelayTxError;

#[cfg(feature = "migration")]
use sc_wallet::MigrateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Max voting is set to > 7 years
const MAX_MULTISIG_VOTING_PERIOD: Duration = Duration::Time(2 << 27);

// set resasonobly high value to not interfere with multisigs
/// Used to spot an multisig instantiate reply
const MULTISIG_INSTANTIATE_ID: u64 = u64::MAX;

#[cfg_attr(not(any(feature = "library", feature = "migration")), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    FROZEN.save(deps.storage, &false)?;
    FACTORY.save(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_ref())?,
    )?;
    CODE_ID.save(deps.storage, &msg.code_id)?;
    MULTISIG_CODE_ID.save(deps.storage, &msg.multisig_code_id)?;

    // get user addr from it's pubkey
    let addr_human = pub_key_to_address(&deps, &msg.create_wallet_msg.user_pubkey.0)?;

    let addr = deps.api.addr_canonicalize(addr_human.as_str())?;

    USER.save(deps.storage, &User { addr, nonce: 0 })?;

    let guardian_addresses = &msg.create_wallet_msg.guardians.addresses;

    for guardian in guardian_addresses {
        let guardian = deps.api.addr_canonicalize(guardian)?;
        GUARDIANS.save(deps.storage, &guardian, &())?;
    }

    for relayer in msg.create_wallet_msg.relayers.iter() {
        let relayer = deps.api.addr_canonicalize(relayer)?;
        RELAYERS.save(deps.storage, &relayer, &())?;
    }

    // Instantiates a cw3 multisig contract if multisig option is provided for guardians
    let resp = if let Some(multisig) = msg.create_wallet_msg.guardians.guardians_multisig {
        let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
            voters: addresses_to_voters(guardian_addresses),
            threshold: Threshold::AbsoluteCount {
                weight: multisig.threshold_absolute_count,
            },
            max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
        };

        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(msg.factory.to_string()),
            code_id: msg.multisig_code_id,
            msg: to_binary(&multisig_instantiate_msg)?,
            funds: multisig.multisig_initial_funds,
            label: "Wallet-Multisig".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, MULTISIG_INSTANTIATE_ID);
        Response::new().add_submessage(msg)
    } else {
        Response::default()
    };

    Ok(resp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Execute { msgs } => execute_execute(deps, env, info, msgs),
        ExecuteMsg::Relay { transaction } => execute_relay(deps, info, transaction),
        ExecuteMsg::RevertFreezeStatus {} => execute_revert_freeze_status(deps, info),
        ExecuteMsg::RotateUserKey { new_user_address } => {
            execute_rotate_user_key(deps, info, new_user_address)
        }
        ExecuteMsg::AddRelayer {
            new_relayer_address,
        } => execute_add_relayer(deps, info, new_relayer_address),
        ExecuteMsg::RemoveRelayer { relayer_address } => {
            execute_remove_relayer(deps, info, relayer_address)
        }
    }
}

/// Executes message from the user
pub fn execute_execute<T>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<T>>,
) -> Result<Response<T>, ContractError>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    // Ensure user exists
    ensure_is_user(deps.as_ref(), info.sender.as_ref())?;

    let res = Response::new()
        .add_messages(msgs)
        .add_attribute("action", "execute");
    Ok(res)
}

/// Executes relayed message fro a relayer
pub fn execute_relay(
    deps: DepsMut,
    info: MessageInfo,
    transaction: RelayTransaction,
) -> Result<Response, ContractError> {
    // Ensure sender is a relayer
    let relayer_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;
    ensure_is_relayer(deps.as_ref(), &relayer_addr)?;

    // Get user addr from it's pubkey
    let addr = pub_key_to_address(&deps, &transaction.user_pubkey.0)?;

    // Ensure address derived from pub message is the address of existing user
    let user = ensure_is_user(deps.as_ref(), &addr.to_string())?;

    // Ensure provided nonce is correct
    user.ensure_nonces_are_equal(&transaction.nonce)?;

    let is_verified = query_verify_cosmos(&deps, &transaction)?;

    if is_verified {
        let msg: Result<CosmosMsg, _> = cosmwasm_std::from_slice(transaction.message.0.as_slice());
        if let Ok(msg) = msg {
            // Update nonce
            USER.update(deps.storage, |mut user| -> StdResult<_> {
                user.increment_nonce();
                Ok(user)
            })?;

            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "execute_relay"))
        } else {
            Err(ContractError::InvalidMessage {})
        }
    } else {
        Err(ContractError::RelayTxError(
            RelayTxError::SignatureVerificationError {},
        ))
    }
}

/// Add relayer to the relayers set
pub fn execute_add_relayer(
    deps: DepsMut,
    info: MessageInfo,
    relayer_addr: Addr,
) -> Result<Response, ContractError> {
    // Authorize user or guardians
    authorize_user_or_guardians(deps.as_ref(), &info.sender)?;

    // Save a new relayer if it does not exist yet
    let relayer_addr_canonical = deps.api.addr_canonicalize(relayer_addr.as_ref())?;

    if !RELAYERS.has(deps.storage, &relayer_addr_canonical) {
        RELAYERS.save(deps.storage, &relayer_addr_canonical, &())?;
        Ok(Response::new().add_attribute("action", format!("Relayer {:?} added", relayer_addr)))
    } else {
        Err(ContractError::RelayerAlreadyExists {})
    }
}

/// Remove relayer from the relayers set
pub fn execute_remove_relayer(
    deps: DepsMut,
    info: MessageInfo,
    relayer_addr: Addr,
) -> Result<Response, ContractError> {
    // Authorize user or guardians
    authorize_user_or_guardians(deps.as_ref(), &info.sender)?;

    // Remove a relayer if possible
    let relayer_addr_canonical = deps.api.addr_canonicalize(relayer_addr.as_ref())?;

    if RELAYERS.has(deps.storage, &relayer_addr_canonical) {
        RELAYERS.remove(deps.storage, &relayer_addr_canonical);

        Ok(Response::new().add_attribute("action", format!("Relayer {:?} removed", relayer_addr)))
    } else {
        Err(ContractError::RelayerDoesNotExist {})
    }
}

/// Change current freezing status to its inverse
/// Must be from a guardian or a guardian multisig contract
pub fn execute_revert_freeze_status(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_canonicalize(info.sender.as_ref())?;

    // Ensure caller is guardian or multisig
    authorize_guardian_or_multisig(deps.as_ref(), &sender)?;

    // Invert frozen status
    let frozen = FROZEN.update(deps.storage, |mut frozen| -> StdResult<_> {
        frozen ^= true;
        Ok(frozen)
    })?;

    let res = Response::new().add_attribute("action", if frozen { "frozen" } else { "unfrozen" });

    Ok(res)
}

/// Complete user key rotation by changing it's address
/// Must be from a guardian or a guardian multisig contract
pub fn execute_rotate_user_key(
    deps: DepsMut,
    info: MessageInfo,
    new_user_address: String,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_canonicalize(info.sender.as_ref())?;

    // Ensure caller is guardian or multisig
    authorize_guardian_or_multisig(deps.as_ref(), &sender)?;

    // Ensure provided address is different from current
    let new_user_address = deps.api.addr_canonicalize(new_user_address.as_ref())?;
    USER.load(deps.storage)?
        .ensure_addresses_are_not_equal(&new_user_address)?;

    // Update user address
    USER.update(deps.storage, |mut user| -> StdResult<_> {
        user.set_address(new_user_address);
        Ok(user)
    })?;

    Ok(Response::new().add_attribute("action", "execute_rotate_user_key"))
}

/// Ensures sender is guardian
pub fn ensure_is_guardian(deps: Deps, sender: &CanonicalAddr) -> Result<(), ContractError> {
    if is_guardian(deps, sender)? {
        Ok(())
    } else {
        Err(ContractError::IsNotGuardian {})
    }
}

/// Is used to authorize guardian or multisig contract
pub fn authorize_guardian_or_multisig(
    deps: Deps,
    sender: &CanonicalAddr,
) -> Result<(), ContractError> {
    match MULTISIG_ADDRESS.may_load(deps.storage)? {
        // if multisig is set, ensure it's address equal to the caller address
        Some(multisig_address) if multisig_address.eq(sender) => Ok(()),
        Some(_) => Err(ContractError::IsNotMultisig {}),
        // if multisig is not set, ensure caller address is guardian
        _ => ensure_is_guardian(deps, sender),
    }
}

/// Is used to authorize user or guardians
pub fn authorize_user_or_guardians(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    let addr_canonical = deps.api.addr_canonicalize(sender.as_ref())?;
    match MULTISIG_ADDRESS.may_load(deps.storage)? {
        // if multisig adrdess is set, check whether sender is equal to it
        Some(multisig_address) if multisig_address.eq(&addr_canonical) => Ok(()),
        // otherwise do user or guardian auth
        _ => {
            let is_user_result = ensure_is_user(deps, sender.as_ref());
            // either guardian or multisig.
            let is_guardian_result = authorize_guardian_or_multisig(deps, &addr_canonical);
            if is_user_result.is_ok() || is_guardian_result.is_ok() {
                Ok(())
            } else {
                is_user_result?;
                is_guardian_result?;
                Ok(())
            }
        }
    }
}

/// Ensures sender is relayer
pub fn ensure_is_relayer(deps: Deps, sender: &CanonicalAddr) -> Result<(), ContractError> {
    if is_relayer(deps, sender)? {
        Ok(())
    } else {
        Err(ContractError::IsNotRelayer {})
    }
}

/// Ensures sender is the wallet user
pub fn ensure_is_user(deps: Deps, sender: &str) -> Result<User, ContractError> {
    let registered_user = USER.load(deps.storage)?;
    let s = deps.api.addr_canonicalize(sender)?;

    if registered_user.addr == s {
        Ok(registered_user)
    } else {
        Err(ContractError::RelayTxError(RelayTxError::IsNotUser {}))
    }
}

/// Checks if the sender is a guardian
fn is_guardian(deps: Deps, sender: &CanonicalAddr) -> StdResult<bool> {
    GUARDIANS
        .may_load(deps.storage, sender)
        .map(|cfg| cfg.is_some())
}

/// Checks if the sender is a relayer
fn is_relayer(deps: Deps, sender: &CanonicalAddr) -> StdResult<bool> {
    RELAYERS
        .may_load(deps.storage, sender)
        .map(|cfg| cfg.is_some())
}

// This is currently just letting guardians execute
fn is_frozen(deps: Deps) -> StdResult<bool> {
    FROZEN.load(deps.storage)
}

// Used to handle different multisig actions
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.result {
        // when new wallet with guardians multisig support instantiated
        ContractResult::Ok(response) if msg.id == MULTISIG_INSTANTIATE_ID => {
            // Note: This is the default instantiate event
            let addr_str = &response.events[0].attributes[0].value;
            let multisig_addr: CanonicalAddr = deps.api.addr_canonicalize(addr_str)?;

            MULTISIG_ADDRESS.save(deps.storage, &multisig_addr)?;

            let res = Response::new()
                .add_attribute("action", "Fixed Multisig Stored")
                .add_attribute("multisig_address", addr_str);
            Ok(res)
        }
        ContractResult::Err(e) => Err(StdError::GenericErr { msg: e }),
        _ => Ok(Response::default()),
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::CanExecuteRelay { sender } => to_binary(&query_can_execute_relay(deps, sender)?),
    }
}

/// Load addresses from store
pub fn load_addresses(deps: &Deps, addresses: Map<&[u8], ()>) -> StdResult<Vec<Addr>> {
    addresses
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key)))
        .collect()
}

/// Load canonical addresses from store
pub fn load_canonical_addresses(deps: &Deps, addresses: Map<&[u8], ()>) -> BTreeSet<Vec<u8>> {
    addresses
        .keys(deps.storage, None, None, Order::Ascending)
        .collect()
}

/// Returns wallet info
pub fn query_info(deps: Deps) -> StdResult<WalletInfo> {
    let guardians = load_addresses(&deps, GUARDIANS)?;
    let relayers = load_addresses(&deps, RELAYERS)?;
    let multisig_address =
        if let Some(multisig_address) = MULTISIG_ADDRESS.may_load(deps.storage)? {
            Some(deps.api.addr_humanize(&multisig_address)?)
        } else {
            None
        };

    let user = USER.load(deps.storage)?;

    Ok(WalletInfo {
        user_addr: deps.api.addr_humanize(&user.addr)?,
        nonce: user.nonce,
        version: cw2::get_contract_version(deps.storage)?,
        code_id: CODE_ID.load(deps.storage)?,
        multisig_code_id: MULTISIG_CODE_ID.load(deps.storage)?,
        guardians,
        relayers,
        is_frozen: FROZEN.load(deps.storage)?,
        multisig_address,
    })
}

/// Returns if sender is a relayer
pub fn query_can_execute_relay(deps: Deps, sender: String) -> StdResult<CanExecuteResponse> {
    let sender_canonical = deps.api.addr_canonicalize(&sender)?;
    Ok(CanExecuteResponse {
        can_execute: is_relayer(deps, &sender_canonical)?,
    })
}

#[cfg(feature = "migration")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        MigrateMsg::Proxy(msg) => {
            CODE_ID.save(deps.storage, &msg.new_code_id)?;
            Ok(Response::default())
        }
        MigrateMsg::Multisig(msg) => {
            // If guardians set changed
            let new_guardians = if let Some(new_guardians) = &msg.new_guardians {
                let guardians = load_canonical_addresses(&deps.as_ref(), GUARDIANS);
                let new_guardian_addresses: Result<BTreeSet<Vec<u8>>, _> = new_guardians
                    .addresses
                    .iter()
                    // CanonicalAddr does not support Ord trait ;(
                    .map(|guardian| {
                        deps.api
                            .addr_canonicalize(guardian)
                            .map(|guardian_addr| guardian_addr.as_slice().to_vec())
                    })
                    .collect();
                let new_guardian_addresses = new_guardian_addresses?;

                // Guardians to remove from storage
                let guardians_to_remove: Vec<_> = guardians
                    .difference(&new_guardian_addresses)
                    .cloned()
                    .collect();

                for guardian in guardians_to_remove {
                    GUARDIANS.remove(deps.storage, &guardian);
                }

                // Guardians to insert into storage
                let guardians_to_insert: Vec<_> = new_guardian_addresses
                    .difference(&guardians)
                    .cloned()
                    .collect();

                for guardian in guardians_to_insert {
                    GUARDIANS.save(deps.storage, &guardian, &())?;
                }

                Some(new_guardians)
            } else {
                None
            };

            let resp = if let Some(new_guardians) = new_guardians {
                // Instantiates a cw3 multisig contract if multisig option is provided for guardians
                if let Some(multisig) = &new_guardians.guardians_multisig {
                    let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
                        voters: addresses_to_voters(&new_guardians.addresses),
                        threshold: Threshold::AbsoluteCount {
                            weight: multisig.threshold_absolute_count,
                        },
                        max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
                    };

                    let instantiate_msg = WasmMsg::Instantiate {
                        admin: Some(FACTORY.load(deps.storage)?.to_string()),
                        code_id: msg.new_multisig_code_id,
                        msg: to_binary(&multisig_instantiate_msg)?,
                        funds: multisig.multisig_initial_funds.clone(),
                        label: "Wallet-Multisig".into(),
                    };
                    let msg = SubMsg::reply_always(instantiate_msg, MULTISIG_INSTANTIATE_ID);
                    Response::new().add_submessage(msg)
                } else {
                    // Unset multisig address if guardians multisig was not provided
                    MULTISIG_ADDRESS.remove(deps.storage);
                    Response::default()
                }
            } else {
                let guardians_str: Vec<String> = load_addresses(&deps.as_ref(), GUARDIANS)?
                    .into_iter()
                    .map(|guardian| guardian.as_str().to_owned())
                    .collect();

                let threshold_response = deps.querier.query_wasm_smart(
                    deps.api.addr_humanize(&MULTISIG_ADDRESS.load(deps.storage)?)?,
                    &FixedMultisigQueryMsg::Threshold {},
                )?;

                if let ThresholdResponse::AbsoluteCount { total_weight, .. } = threshold_response {
                    // Upgrade to a new multisig contract
                    let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
                        voters: addresses_to_voters(&guardians_str),
                        threshold: Threshold::AbsoluteCount {
                            // reuse previous multisig contract state
                            weight: total_weight,
                        },
                        max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
                    };

                    let instantiate_msg = WasmMsg::Instantiate {
                        admin: Some(FACTORY.load(deps.storage)?.to_string()),
                        code_id: msg.new_multisig_code_id,
                        msg: to_binary(&multisig_instantiate_msg)?,
                        funds: vec![],
                        label: "Wallet-Multisig".into(),
                    };
                    let msg = SubMsg::reply_always(instantiate_msg, MULTISIG_INSTANTIATE_ID);
                    Response::new().add_submessage(msg)
                } else {
                    // Should be set to ThresholdResponse::AbsoluteCount
                    return Err(ContractError::IncorrectThreshold {})
                }
            };

            MULTISIG_CODE_ID.save(deps.storage, &msg.new_multisig_code_id)?;

            let code_id = MULTISIG_CODE_ID.load(deps.storage);
            println!("{:?} is stored", code_id);
            Ok(resp)
        }
    }
}

// Converts addresses to voters with weight of 1
pub fn addresses_to_voters(addresses: &[String]) -> Vec<Voter> {
    addresses
        .iter()
        .map(|address| Voter {
            addr: address.to_owned(),
            weight: 1,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, BankMsg, DepsMut};

    use crate::contract::instantiate;
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg};

    use sc_wallet::{CreateWalletMsg, Guardians};
    use secp256k1::bitcoin_hashes::sha256;
    use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};

    const GUARD1: &str = "guardian1";
    const GUARD2: &str = "guardian2";

    fn get_guardians() -> Guardians {
        Guardians {
            addresses: vec![GUARD1.to_string(), GUARD2.to_string()],
            guardians_multisig: None,
        }
    }

    const RELAYER1: &str = "relayer1";
    const RELAYER2: &str = "relayer2";
    const RELAYER3: &str = "relayer3";

    const INVALID_GUARD: &str = "not_a_guardian";

    const INVALID_RELAYER: &str = "not_a_relayer";

    const USER_PRIV: &[u8; 32] = &[
        239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
        185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
    ];

    const INVAILID_USER_PRIV: &[u8; 32] = &[
        239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
        185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 27,
    ];

    const MULTISIG_CODE_ID: u64 = 13;

    // this will set up the instantiation for other tests
    // returns User address
    fn do_instantiate(mut deps: DepsMut) -> Addr {
        let secp = Secp256k1::new();

        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_serialized = Binary(public_key.serialize_uncompressed().to_vec());

        let create_wallet_msg = CreateWalletMsg {
            user_pubkey: public_key_serialized.clone(),
            guardians: get_guardians(),
            relayers: vec![RELAYER1.into(), RELAYER2.into()],
            proxy_initial_funds: vec![],
        };

        let instantiate_msg = InstantiateMsg {
            create_wallet_msg,
            multisig_code_id: MULTISIG_CODE_ID,
            factory: Addr::unchecked("factory"),
            code_id: 0,
        };

        let info = mock_info("creator", &[]);
        let env = mock_env();

        let address = pub_key_to_address(&deps, &public_key_serialized).unwrap();
        instantiate(deps.branch(), env, info, instantiate_msg).unwrap();

        address
    }

    #[test]
    fn guardian_can_revert_freeze_status() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        do_instantiate(deps.as_mut());

        // initially it is not frozen
        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(!wallet_info.is_frozen);

        // GUARD1 is a valid guardian
        let info = mock_info(GUARD1, &[]);
        let env = mock_env();
        let msg = ExecuteMsg::RevertFreezeStatus {};
        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(response.attributes, [("action", format!("frozen"))]);

        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(wallet_info.is_frozen);

        let msg = ExecuteMsg::RevertFreezeStatus {};
        let response = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(response.attributes, [("action", format!("unfrozen"))]);

        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(!wallet_info.is_frozen);
    }

    #[test]
    fn non_guardian_cannot_revert_freeze_status() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        do_instantiate(deps.as_mut());

        // INVALID_GUARD is not a valid guardian
        let info = mock_info(INVALID_GUARD, &[]);
        let env = mock_env();
        let msg = ExecuteMsg::RevertFreezeStatus {};
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::IsNotGuardian {});
    }

    #[test]
    fn user_can_add_relayer() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let user_addr = do_instantiate(deps.as_mut());

        // initially we have a wallet with 2 relayers
        let mut wallet_info = query_info(deps.as_ref()).unwrap();

        let info = mock_info(user_addr.as_str(), &[]);
        let env = mock_env();

        let new_relayer_address = Addr::unchecked(RELAYER3);
        let msg = ExecuteMsg::AddRelayer {
            new_relayer_address: new_relayer_address.clone(),
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            response.attributes,
            [("action", format!("Relayer {:?} added", new_relayer_address))]
        );

        // Ensure relayer is added successfully
        wallet_info.relayers.push(new_relayer_address);
        let new_wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(new_wallet_info.relayers == new_wallet_info.relayers);
    }

    #[test]
    fn add_relayer() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let user_addr = do_instantiate(deps.as_mut());

        // initially we have a wallet with 2 relayers
        let mut wallet_info = query_info(deps.as_ref()).unwrap();

        let info = mock_info(user_addr.as_str(), &[]);
        let env = mock_env();

        let new_relayer_address = Addr::unchecked(RELAYER3);
        let msg = ExecuteMsg::AddRelayer {
            new_relayer_address: new_relayer_address.clone(),
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            response.attributes,
            [("action", format!("Relayer {:?} added", new_relayer_address))]
        );

        // Ensure relayer is added successfully
        wallet_info.relayers.push(new_relayer_address);
        let new_wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(new_wallet_info.relayers == new_wallet_info.relayers);
    }

    #[test]
    fn user_can_remove_relayer() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let user_addr = do_instantiate(deps.as_mut());

        // initially we have a wallet with 2 relayers
        let mut wallet_info = query_info(deps.as_ref()).unwrap();

        let info = mock_info(user_addr.as_str(), &[]);
        let env = mock_env();

        let relayer_address = Addr::unchecked(RELAYER2);
        let msg = ExecuteMsg::RemoveRelayer {
            relayer_address: relayer_address.clone(),
        };

        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            response.attributes,
            [("action", format!("Relayer {:?} removed", relayer_address))]
        );

        // Ensure relayer is removed successfully
        wallet_info.relayers = wallet_info
            .relayers
            .into_iter()
            .filter(|relayer| *relayer == relayer_address)
            .collect();
        let new_wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(new_wallet_info.relayers == new_wallet_info.relayers);
    }

    #[test]
    fn guardian_can_rotate_user_key() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        do_instantiate(deps.as_mut());

        // initially it is not frozen
        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(!wallet_info.is_frozen);

        // GUARD1 is a valid guardian
        let info = mock_info(GUARD1, &[]);
        let env = mock_env();

        let new_address = "new_key";
        let msg = ExecuteMsg::RotateUserKey {
            new_user_address: new_address.to_string(),
        };
        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            response.attributes,
            [("action", format!("execute_rotate_user_key"))]
        );

        // Ensure key is rotated successfully
        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(new_address.eq(wallet_info.user_addr.as_str()));
    }

    #[test]
    fn non_guardian_cannot_rotate_user_key() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        do_instantiate(deps.as_mut());

        // INVALID_GUARD is not a valid guardian
        let info = mock_info(INVALID_GUARD, &[]);
        let env = mock_env();

        let new_address = "new_key";
        let msg = ExecuteMsg::RotateUserKey {
            new_user_address: new_address.to_string(),
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::IsNotGuardian {});
    }

    #[test]
    fn rotate_user_key_same_address() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let user_addr = do_instantiate(deps.as_mut());

        // initially it is not frozen
        let wallet_info = query_info(deps.as_ref()).unwrap();
        assert!(!wallet_info.is_frozen);

        // GUARD1 is a valid guardian
        let info = mock_info(GUARD1, &[]);
        let env = mock_env();

        // Make an attempt to provide the same address as currently set
        let msg = ExecuteMsg::RotateUserKey {
            new_user_address: user_addr.to_string(),
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::AddressesAreEqual {});
    }

    #[test]
    fn relay_proxy_user_tx() {
        let coins = coins(2, "token");

        let mut deps = mock_dependencies_with_balance(&coins);
        do_instantiate(deps.as_mut());

        // RELAYER1 is a valid relayer
        let info = mock_info(RELAYER1, &[]);
        let nonce = query_info(deps.as_ref()).unwrap().nonce;

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let bank_msg = BankMsg::Burn { amount: coins };

        let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        let relay_transaction = RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            user_pubkey: Binary(public_key.serialize().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        };

        let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap();

        assert_eq!(response.attributes, [("action", "execute_relay")]);
    }

    #[test]
    fn relay_proxy_user_tx_invalid_msg() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        do_instantiate(deps.as_mut());

        // RELAYER1 is a valid relayer
        let info = mock_info(RELAYER1, &[]);
        let nonce = query_info(deps.as_ref()).unwrap().nonce;

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let msg_slice = [0xab; 32];

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_slice
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        let relay_transaction = RelayTransaction {
            message: Binary(msg_slice.to_vec()),
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        };

        let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

        assert_eq!(response, ContractError::InvalidMessage {});
    }

    #[test]
    fn relay_proxy_user_tx_is_not_relayer() {
        let coins = coins(2, "token");

        let mut deps = mock_dependencies_with_balance(&coins);
        do_instantiate(deps.as_mut());

        // INVALID_RELAYER is not a valid relayer
        let info = mock_info(INVALID_RELAYER, &[]);
        let nonce = query_info(deps.as_ref()).unwrap().nonce;

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let bank_msg = BankMsg::Burn { amount: coins };

        let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        let relay_transaction = RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        };

        let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

        assert_eq!(response, ContractError::IsNotRelayer {});
    }

    #[test]
    fn relay_proxy_user_tx_is_not_user() {
        let coins = coins(2, "token");

        let mut deps = mock_dependencies_with_balance(&coins);
        do_instantiate(deps.as_mut());

        // RELAYER1 is a valid relayer
        let info = mock_info(RELAYER1, &[]);
        let nonce = query_info(deps.as_ref()).unwrap().nonce;

        // Make an attempt to relay message of nonexistent user
        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_slice(INVAILID_USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let bank_msg = BankMsg::Burn { amount: coins };

        let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        let relay_transaction = RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        };

        let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

        assert_eq!(
            response,
            ContractError::RelayTxError(RelayTxError::IsNotUser {})
        );
    }

    #[test]
    fn relay_proxy_user_tx_invalid_nonce() {
        let coins = coins(2, "token");

        let mut deps = mock_dependencies_with_balance(&coins);
        do_instantiate(deps.as_mut());

        // RELAYER1 is a valid relayer
        let info = mock_info(RELAYER1, &[]);
        let nonce = query_info(deps.as_ref()).unwrap().nonce;

        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let bank_msg = BankMsg::Burn { amount: coins };

        let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        let relay_transaction = RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            user_pubkey: Binary(public_key.serialize().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce: nonce + 1,
        };

        let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

        assert_eq!(
            response,
            ContractError::RelayTxError(RelayTxError::NoncesAreNotEqual {})
        );
    }
}
