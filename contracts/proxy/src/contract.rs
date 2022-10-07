#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
use schemars::JsonSchema;
use std::fmt;
use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, GuardiansUpdateMsg,
    GuardiansUpdateRequest, RelayTransaction, RelayTxError, WalletFactoryExecuteMsg,
    WalletFactoryQueryMsg, WalletInfo,
};

use crate::error::ContractError;
use crate::helpers::{
    addresses_to_voters, authorize_guardian_or_multisig, authorize_user_or_guardians,
    ensure_is_contract_self, ensure_is_relayer, ensure_is_relayer_or_user, ensure_is_user,
    is_frozen, is_relayer, load_addresses, load_canonical_addresses,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    User, ADDR_PREFIX, CODE_ID, FACTORY, FROZEN, GUARDIANS, GUARDIANS_UPDATE_REQUEST, LABEL,
    MULTISIG_ADDRESS, MULTISIG_CODE_ID, RELAYERS, USER,
};
use cw3_fixed_multisig::msg::InstantiateMsg as FixedMultisigInstantiateMsg;
use cw_utils::{parse_reply_instantiate_data, Duration, Threshold};

#[cfg(feature = "migration")]
use vectis_wallet::ProxyMigrateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Max voting is set to > 7 years
const MAX_MULTISIG_VOTING_PERIOD: Duration = Duration::Time(2 << 27);

// set resasonobly high value to not interfere with multisigs
/// Used to spot an multisig instantiate reply
const MULTISIG_INSTANTIATE_ID: u64 = u64::MAX;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Ensure no guardians are the same as a user
    // https://github.com/nymlab/vectis/issues/43
    let addr_human = deps.api.addr_validate(&msg.create_wallet_msg.user_addr)?;
    msg.create_wallet_msg
        .guardians
        .verify_guardians(&addr_human)?;

    let addr = deps.api.addr_canonicalize(addr_human.as_str())?;
    USER.save(
        deps.storage,
        &User {
            addr: addr.clone(),
            nonce: 0,
        },
    )?;

    FROZEN.save(deps.storage, &false)?;
    FACTORY.save(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_ref())?,
    )?;
    CODE_ID.save(deps.storage, &msg.code_id)?;
    MULTISIG_CODE_ID.save(deps.storage, &msg.multisig_code_id)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    LABEL.save(deps.storage, &msg.create_wallet_msg.label)?;

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
            admin: Some(env.contract.address.to_string()),
            code_id: msg.multisig_code_id,
            msg: to_binary(&multisig_instantiate_msg)?,
            funds: multisig.multisig_initial_funds,
            label: "Wallet-Multisig".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, MULTISIG_INSTANTIATE_ID);
        Response::new()
            .add_submessage(msg)
            .add_attribute("user", addr_human)
            .set_data(addr.0)
    } else {
        Response::new()
            .add_attribute("user", addr_human)
            .set_data(addr.0)
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
        ExecuteMsg::RequestUpdateGuardians { request } => {
            execute_request_update_guardians(deps, info, env, request)
        }
        ExecuteMsg::UpdateGuardians {} => execute_update_guardians(deps, env, info),
        ExecuteMsg::UpdateLabel { new_label } => execute_update_label(deps, info, env, new_label),
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

    // make sure guardians have not frozen the contract
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    // Get user addr from it's pubkey
    let addr = pub_key_to_address(
        &deps.as_ref(),
        &ADDR_PREFIX.load(deps.storage)?,
        &transaction.user_pubkey.0,
    )?;

    // Ensure address derived from pub message is the address of existing user
    let user = ensure_is_user(deps.as_ref(), &addr.to_string())?;

    // Ensure relayer provided nonce is correct
    user.ensure_nonces_are_equal(&transaction.nonce)?;

    // Checks signature, which includes a message including the nonce signed by the user
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
            Err(ContractError::InvalidMessage {
                msg: msg.unwrap_err().to_string(),
            })
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
    // Ensure caller is guardian or multisig
    authorize_guardian_or_multisig(deps.as_ref(), &info.sender)?;

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
    // Allow guardians to rotate key when it is frozen
    if is_frozen(deps.as_ref())?
        && authorize_guardian_or_multisig(deps.as_ref(), &info.sender).is_err()
    {
        return Err(ContractError::Frozen {});
    } else {
        authorize_user_or_guardians(deps.as_ref(), &info.sender)?
    };

    let user = USER.load(deps.storage)?;
    let update_factory = WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&FACTORY.load(deps.storage)?)?
            .to_string(),
        /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
        msg: to_binary(&WalletFactoryExecuteMsg::UpdateProxyUser {
            old_user: deps.api.addr_humanize(&user.addr)?,
            new_user: deps.api.addr_validate(&new_user_address)?,
        })?,
        funds: vec![],
    };
    let msg = SubMsg::<Empty>::new(update_factory);

    // Ensure provided address is different from current
    let new_user_address = deps.api.addr_canonicalize(new_user_address.as_ref())?;
    user.ensure_addresses_are_not_equal(&new_user_address)?;

    // Update user address
    USER.update(deps.storage, |mut user| -> StdResult<_> {
        user.set_address(new_user_address);
        Ok(user)
    })?;

    Ok(Response::new()
        .add_submessage(msg)
        .add_attribute("action", "execute_rotate_user_key"))
}

pub fn execute_update_guardians(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    ensure_is_relayer_or_user(deps.as_ref(), &env, &info.sender)?;

    // make sure guardians have not frozen the contract
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    match GUARDIANS_UPDATE_REQUEST
        .may_load(deps.storage)?
        .ok_or(ContractError::GuardianRequestNotFound {})?
    {
        Some(request) => {
            if !request.activate_at.is_expired(&env.block) {
                return Err(ContractError::GuardianRequestNotExecutable {});
            }

            let GuardiansUpdateRequest {
                guardians,
                new_multisig_code_id,
                ..
            } = request;

            // Replace the entire locally stored guardians list
            let guardians_to_remove = load_canonical_addresses(&deps.as_ref(), GUARDIANS)?;
            for guardian in guardians_to_remove {
                GUARDIANS.remove(deps.storage, &guardian);
            }
            for guardian in &guardians.addresses {
                GUARDIANS.save(deps.storage, &deps.api.addr_canonicalize(guardian)?, &())?;
            }

            if let Some(multisig_settings) = guardians.guardians_multisig {
                let instantiation_code_id = if let Some(id) = new_multisig_code_id {
                    id
                } else {
                    match MULTISIG_CODE_ID.may_load(deps.storage)? {
                        Some(id) => id,
                        None => deps.querier.query_wasm_smart(
                            deps.api.addr_humanize(&FACTORY.load(deps.storage)?)?,
                            &{
                                WalletFactoryQueryMsg::CodeId {
                                    ty: CodeIdType::Multisig,
                                }
                            },
                        )?,
                    }
                };
                MULTISIG_CODE_ID.save(deps.storage, &instantiation_code_id)?;
                let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
                    voters: addresses_to_voters(&guardians.addresses),
                    threshold: Threshold::AbsoluteCount {
                        weight: multisig_settings.threshold_absolute_count,
                    },
                    max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
                };

                let instantiate_msg = WasmMsg::Instantiate {
                    admin: Some(env.contract.address.to_string()),
                    code_id: instantiation_code_id,
                    msg: to_binary(&multisig_instantiate_msg)?,
                    funds: multisig_settings.multisig_initial_funds,
                    label: "Wallet-Multisig".into(),
                };
                let msg = SubMsg::reply_always(instantiate_msg, MULTISIG_INSTANTIATE_ID);

                GUARDIANS_UPDATE_REQUEST.save(deps.storage, &None)?;

                Ok(Response::new()
                    .add_submessage(msg)
                    .add_attribute("action", "Updated wallet guardians: Multisig"))
            } else {
                MULTISIG_ADDRESS.save(deps.storage, &None)?;
                GUARDIANS_UPDATE_REQUEST.save(deps.storage, &None)?;
                Ok(Response::new()
                    .add_attribute("action", "Updated wallet guardians: Non-Multisig"))
            }
        }
        None => Err(ContractError::GuardianRequestNotFound {}),
    }
}

pub fn execute_request_update_guardians(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    request: Option<GuardiansUpdateMsg>,
) -> Result<Response, ContractError> {
    ensure_is_relayer_or_user(deps.as_ref(), &env, &info.sender)?;

    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    match request {
        Some(r) => {
            r.guardians
                .verify_guardians(&deps.api.addr_humanize(&USER.load(deps.storage)?.addr)?)?;

            GUARDIANS_UPDATE_REQUEST.save(
                deps.storage,
                &Some(GuardiansUpdateRequest::new(
                    r.guardians,
                    r.new_multisig_code_id,
                    &env.block,
                )),
            )?;
        }
        None => GUARDIANS_UPDATE_REQUEST.save(deps.storage, &None)?,
    }

    Ok(Response::new().add_attribute("action", "Update guardians request created"))
}

/// Update label by user
pub fn execute_update_label(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    new_label: String,
) -> Result<Response, ContractError> {
    let is_user = ensure_is_user(deps.as_ref(), info.sender.as_str());
    let is_contract = ensure_is_contract_self(&env, &info.sender);
    if is_user.is_err() && is_contract.is_err() {
        is_user?;
        is_contract?;
    }

    LABEL.update(deps.storage, |l| {
        if l == new_label {
            Err(ContractError::SameLabel {})
        } else {
            Ok(new_label.clone())
        }
    })?;
    Ok(Response::default()
        .add_attribute("action", "update label")
        .add_attribute("label", new_label))
}

// Used to handle different multisig actions
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id == MULTISIG_INSTANTIATE_ID {
        if let Ok(res) = parse_reply_instantiate_data(reply) {
            MULTISIG_ADDRESS.save(
                deps.storage,
                &Some(deps.api.addr_canonicalize(&res.contract_address)?),
            )?;

            Ok(Response::new()
                .add_attribute("action", "Fixed Multisig Stored")
                .add_attribute("multisig_address", res.contract_address))
        } else {
            Err(ContractError::MultisigInstantiationError {})
        }
    } else {
        Err(ContractError::InvalidMessage {
            msg: "invalid ID".to_string(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::CanExecuteRelay { sender } => to_binary(&query_can_execute_relay(deps, sender)?),
        QueryMsg::GuardiansUpdateRequest {} => to_binary(&query_guardian_update_request(deps)?),
    }
}

/// Returns wallet info
pub fn query_info(deps: Deps) -> StdResult<WalletInfo> {
    let guardians = load_addresses(&deps, GUARDIANS)?;
    let relayers = load_addresses(&deps, RELAYERS)?;
    let multisig_address = match MULTISIG_ADDRESS.may_load(deps.storage)? {
        Some(Some(c)) => Some(deps.api.addr_humanize(&c)?),
        _ => None,
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
        label: LABEL.load(deps.storage)?,
    })
}

/// Returns if sender is a relayer
pub fn query_can_execute_relay(deps: Deps, sender: String) -> StdResult<CanExecuteResponse> {
    let sender_canonical = deps.api.addr_canonicalize(&sender)?;
    Ok(CanExecuteResponse {
        can_execute: is_relayer(deps, &sender_canonical)?,
    })
}

pub fn query_guardian_update_request(deps: Deps) -> StdResult<Option<GuardiansUpdateRequest>> {
    match GUARDIANS_UPDATE_REQUEST.may_load(deps.storage)? {
        Some(r) => Ok(r),
        None => Ok(None),
    }
}

#[cfg(feature = "migration")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: ProxyMigrateMsg) -> Result<Response, ContractError> {
    CODE_ID.save(deps.storage, &msg.new_code_id)?;
    Ok(Response::default())
}
