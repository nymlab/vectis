use crate::error::{ContractError, MigrationMsgError, RelayMigrationError};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use crate::state::{ADMIN, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID, TOTAL_CREATED, WALLETS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, ContractResult, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Reply, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
pub use sc_wallet::{
    pub_key_to_address, query_verify_cosmos, CreateWalletMsg, Guardians, ProxyMigrationMsg,
    RelayTransaction, WalletAddr, WalletInfo,
};
use wallet_proxy::msg::{InstantiateMsg as ProxyInstantiateMsg, QueryMsg as ProxyQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;

    ADMIN.save(deps.storage, &admin_addr)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id)?;
    PROXY_MULTISIG_CODE_ID.save(deps.storage, &msg.proxy_multisig_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateWallet { create_wallet_msg } => {
            create_wallet(deps, env, info, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            proxy_migration_msg,
        } => migrate_wallet(deps, info, wallet_address, proxy_migration_msg),
        ExecuteMsg::UpdateProxyCodeId { new_code_id } => {
            update_proxy_code_id(deps, info, new_code_id)
        }
    }
}

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    // Only admin can create a new wallet
    // This can be an account or a governing contract
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;

    if create_wallet_msg.guardians.addresses.is_empty() {
        return Err(ContractError::EmptyGuardians {});
    }

    // Ensure fixed multisig threshold is valid, if provided
    ensure_is_valid_threshold(&create_wallet_msg.guardians)?;

    if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(1) {
        // The wasm message containing the `wallet_proxy` instantiation message
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: PROXY_CODE_ID.load(deps.storage)?,
            msg: to_binary(&ProxyInstantiateMsg {
                factory: env.contract.address,
                multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                create_wallet_msg: create_wallet_msg.clone(),
                code_id: PROXY_CODE_ID.load(deps.storage)?,
            })?,
            funds: create_wallet_msg.proxy_initial_funds,
            label: "Wallet-Proxy".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, next_id);
        let res = Response::new().add_submessage(msg);
        TOTAL_CREATED.save(deps.storage, &next_id)?;
        Ok(res)
    } else {
        Err(ContractError::OverFlow {})
    }
}

/// Migrates the instantiated `wallet_proxy` instance to a new code id
fn migrate_wallet(
    deps: DepsMut,
    info: MessageInfo,
    address: WalletAddr,
    proxy_migration_msg: ProxyMigrationMsg,
) -> Result<Response, ContractError> {
    let wallet_addr = match address {
        WalletAddr::Canonical(canonical_address) => deps.api.addr_humanize(&canonical_address)?,
        WalletAddr::Addr(human_address) => human_address,
    };

    let wallet_info: WalletInfo = deps
        .querier
        .query_wasm_smart(wallet_addr.clone(), &ProxyQueryMsg::Info {})?;

    // The migration call is either directly called by the user with `ProxyMigrationMsg::DirectMigrationMsg`
    // or relayed by the proxy relayer via `ProxyMigrationMsg::RelayTx`.
    //
    // Different safety checks are applied
    let tx_msg: CosmosMsg = match proxy_migration_msg {
        ProxyMigrationMsg::RelayTx(tx) => {
            let can_execute_relay: CanExecuteResponse = deps.querier.query_wasm_smart(
                wallet_addr.clone(),
                &ProxyQueryMsg::CanExecuteRelay {
                    sender: info.sender.to_string(),
                },
            )?;

            // Ensure caller is a wallet relayer
            if !can_execute_relay.can_execute {
                return Err(ContractError::Unauthorized {});
            } else {
                // Ensure Signer of relayed message is the wallet user
                if wallet_info.user_addr != pub_key_to_address(&deps, &tx.user_pubkey.0)? {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayMigrationError::MismatchUserAddr,
                    ));
                };

                // Ensure none of relayed message is the expected next wallet nonce
                if wallet_info.nonce != tx.nonce {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayMigrationError::MismatchNonce,
                    ));
                };

                // Verify signature
                if !query_verify_cosmos(&deps, &tx)? {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayMigrationError::SignatureVerificationError,
                    ));
                };

                cosmwasm_std::from_slice(tx.message.0.as_slice())?
            }
        }
        ProxyMigrationMsg::DirectMigrationMsg(msg) => {
            // Ensure caller is the wallet user
            if wallet_info.user_addr != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            cosmwasm_std::from_slice(&msg)?
        }
    };

    // Further checks applied to ensure user has signed the correct relay msg / tx
    if let CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr,
        new_code_id,
        msg: _,
    }) = tx_msg.clone()
    {
        // Ensure migrating the corret wallet at given address
        if contract_addr != wallet_addr {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::InvalidWalletAddr,
            ));
        }
        // Ensure user knows the latest supported proxy code id
        if new_code_id != PROXY_CODE_ID.load(deps.storage)? || new_code_id == wallet_info.code_id {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::MismatchCodeId,
            ));
        };
    } else {
        return Err(ContractError::InvalidMigrationMsg(
            MigrationMsgError::InvalidWasmMsg,
        ));
    }

    Ok(Response::new()
        .add_message(tx_msg)
        .add_attribute("action", "wallet migration"))
}

/// Updates the latest code id for the supported `wallet_proxy`
fn update_proxy_code_id(
    deps: DepsMut,
    info: MessageInfo,
    new_code_id: u64,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;
    let updated_code_id = PROXY_CODE_ID.update(
        deps.storage,
        |mut current_code_id| -> Result<_, ContractError> {
            if current_code_id != new_code_id {
                current_code_id = new_code_id;
                Ok(current_code_id)
            } else {
                Err(ContractError::SameCodeId {})
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("config", "Proxy Code Id")
        .add_attribute("proxy_code_id", format!("{}", updated_code_id)))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, StdError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert
    let expected_id = TOTAL_CREATED.load(deps.storage)?;
    if msg.id == expected_id {
        match msg.result {
            ContractResult::Ok(response) => {
                // Note: This is the default instantiate event
                let addr_str = &response.events[0].attributes[0].value;
                let proxy_addr: CanonicalAddr = deps.api.addr_canonicalize(addr_str)?;

                WALLETS.save(deps.storage, &proxy_addr, &())?;

                let res = Response::new()
                    .add_attribute("action", "Wallet Proxy Stored")
                    .add_attribute("proxy_address", addr_str);
                Ok(res)
            }
            ContractResult::Err(e) => Err(StdError::GenericErr { msg: e }),
        }
    } else {
        Err(StdError::NotFound {
            kind: "Reply Id".into(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Wallets {} => to_binary(&query_wallet_list(deps)?),
        QueryMsg::ProxyCodeId {} => to_binary(&query_proxy_code_id(deps)?),
    }
}

/// Returns all the wallets created
fn query_wallet_list(deps: Deps) -> StdResult<WalletListResponse> {
    let wallets: Result<Vec<_>, _> = WALLETS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key)))
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
}

/// Returns the current supported `wallet_proxy` code id
fn query_proxy_code_id(deps: Deps) -> StdResult<u64> {
    let id = PROXY_CODE_ID.load(deps.storage)?;
    Ok(id)
}

/// Ensures provided addr is the state stored ADMIN
fn ensure_is_admin(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller == admin {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

/// Ensures provided fixed multisig threshold is valid.
fn ensure_is_valid_threshold(guardians: &Guardians) -> Result<(), ContractError> {
    match &guardians.guardians_multisig {
        Some(g) if g.threshold_absolute_count == 0 => {
            Err(ContractError::ThresholdShouldBeGreaterThenZero {})
        }
        Some(g) if g.threshold_absolute_count > guardians.addresses.len() as u64 => {
            Err(ContractError::ThresholdShouldBeLessThenGuardiansCount {})
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::DepsMut;

    use crate::contract::instantiate;
    use crate::msg::WalletListResponse;

    // this will set up the instantiation for other tests
    fn do_instantiate(mut deps: DepsMut, proxy_code_id: u64, proxy_multisig_code_id: u64) {
        // we do not do integrated tests here so code ids are arbitrary
        let instantiate_msg = InstantiateMsg {
            proxy_code_id,
            proxy_multisig_code_id,
        };
        let info = mock_info("admin", &[]);
        let env = mock_env();

        instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    }

    #[test]
    fn initialise_with_no_wallets() {
        let mut deps = mock_dependencies();

        do_instantiate(deps.as_mut(), 0, 1);

        // no wallets to start
        let wallets: WalletListResponse = query_wallet_list(deps.as_ref()).unwrap();
        assert_eq!(wallets.wallets.len(), 0);
    }

    #[test]
    fn initialise_with_correct_code_id() {
        let mut deps = mock_dependencies();
        let initial_code_id = 1111;
        let initial_multisig_code_id = 2222;
        do_instantiate(deps.as_mut(), initial_code_id, initial_multisig_code_id);
        let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
        assert_eq!(proxy_code_id, initial_code_id);
    }

    #[test]
    fn admin_upgrade_code_id_works() {
        let mut deps = mock_dependencies();
        let initial_code_id = 1111;
        let new_code_id = 2222;
        let initial_multisig_code_id = 2222;
        do_instantiate(deps.as_mut(), initial_code_id, initial_multisig_code_id);
        let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
        assert_eq!(proxy_code_id, initial_code_id);

        let info = mock_info("admin", &[]);
        let env = mock_env();
        let msg = ExecuteMsg::UpdateProxyCodeId { new_code_id };
        let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            response.attributes,
            [
                ("config", "Proxy Code Id"),
                ("proxy_code_id", &new_code_id.to_string())
            ]
        );

        let new_proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
        assert_eq!(new_proxy_code_id, new_code_id);
    }
}
