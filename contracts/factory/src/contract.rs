use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use crate::state::{
    ADDR_PREFIX, ADMIN, COIN_DENOM, FEE, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID, TOTAL_CREATED,
    WALLETS,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
pub use sc_wallet::{
    pub_key_to_address, query_verify_cosmos, CreateWalletMsg, Guardians, MigrationMsgError,
    ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction, RelayTxError, WalletAddr, WalletInfo,
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
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    COIN_DENOM.save(deps.storage, &msg.coin_denom)?;
    FEE.save(deps.storage, &Uint128::from(msg.wallet_fee))?;

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
            create_wallet(deps, env, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            migration_msg,
        } => migrate_wallet(deps, info, wallet_address, migration_msg),
        ExecuteMsg::UpdateProxyCodeId { new_code_id } => {
            update_proxy_code_id(deps, info, new_code_id)
        }
        ExecuteMsg::UpdateProxyMultisigCodeId { new_code_id } => {
            update_proxy_multisig_code_id(deps, info, new_code_id)
        }
        ExecuteMsg::UpdateWalletFee { new_fee } => update_wallet_fee(deps, info, new_fee),
    }
}

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
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
                multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                create_wallet_msg: create_wallet_msg.clone(),
                code_id: PROXY_CODE_ID.load(deps.storage)?,
                addr_prefix: ADDR_PREFIX.load(deps.storage)?,
            })?,
            funds: create_wallet_msg.proxy_initial_funds,
            label: "Wallet-Proxy".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, next_id);
        let res = Response::new().add_submessage(msg);
        TOTAL_CREATED.save(deps.storage, &next_id)?;

        // Transfer tokenFEE.load(deps.storage)?s to the DAO
        let fee = FEE.load(deps.storage)?;
        if fee != Uint128::zero() {
            let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&ADMIN.load(deps.storage)?)?
                    .to_string(),
                amount: vec![Coin {
                    denom: COIN_DENOM.load(deps.storage)?,
                    amount: fee,
                }],
            });
            return Ok(res.add_message(bank_msg));
        }
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
    migration_msg: ProxyMigrationTxMsg,
) -> Result<Response, ContractError> {
    let wallet_addr = match address {
        WalletAddr::Canonical(canonical_address) => deps.api.addr_humanize(&canonical_address)?,
        WalletAddr::Addr(human_address) => human_address,
    };

    let wallet_info: WalletInfo = deps
        .querier
        .query_wasm_smart(wallet_addr.clone(), &ProxyQueryMsg::Info {})?;

    // The migration call is either directly called by the user with `ProxyMigrationTxMsg::DirectMigrationMsg`
    // or relayed by the proxy relayer via `ProxyMigrationTxMsg::RelayTx`.
    //
    // Different safety checks are applied
    let tx_msg: CosmosMsg =
        ensure_is_valid_migration_msg(&deps, info, &wallet_info, &wallet_addr, migration_msg)?;

    // Further checks applied to ensure user has signed the correct relay msg / tx
    if let CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr,
        new_code_id,
        msg,
    }) = tx_msg.clone()
    {
        let msg: ProxyMigrateMsg = cosmwasm_std::from_slice(&msg)?;

        // Ensure user knows the latest supported proxy code id
        msg.ensure_is_supported_proxy_code_id(PROXY_CODE_ID.load(deps.storage)?)?;
        if new_code_id != PROXY_CODE_ID.load(deps.storage)? {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::MismatchProxyCodeId,
            ));
        }

        // Ensure migrating the corret wallet at given address
        if contract_addr != wallet_addr {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::InvalidWalletAddr,
            ));
        }
    } else {
        return Err(ContractError::InvalidMigrationMsg(
            MigrationMsgError::InvalidWasmMsg,
        ));
    }

    Ok(Response::new()
        .add_message(tx_msg)
        .add_attribute("action", "wallet migration"))
}

// Perform security checks to ensure migration message is valid
fn ensure_is_valid_migration_msg(
    deps: &DepsMut,
    info: MessageInfo,
    wallet_info: &WalletInfo,
    wallet_addr: &Addr,
    migration_msg: ProxyMigrationTxMsg,
) -> Result<CosmosMsg, ContractError> {
    let tx_msg: CosmosMsg = match migration_msg {
        ProxyMigrationTxMsg::RelayTx(tx) => {
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
                if wallet_info.user_addr
                    != pub_key_to_address(
                        deps,
                        &ADDR_PREFIX.load(deps.storage)?,
                        &tx.user_pubkey.0,
                    )?
                {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::IsNotUser {},
                    ));
                };

                // Ensure none of relayed message is the expected next wallet nonce
                if wallet_info.nonce != tx.nonce {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::NoncesAreNotEqual {},
                    ));
                };

                // Verify signature
                if !query_verify_cosmos(deps, &tx)? {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::SignatureVerificationError {},
                    ));
                };

                cosmwasm_std::from_slice(tx.message.0.as_slice())?
            }
        }
        ProxyMigrationTxMsg::DirectMigrationMsg(msg) => {
            // Ensure caller is the wallet user
            if wallet_info.user_addr != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            cosmwasm_std::from_slice(&msg)?
        }
    };
    Ok(tx_msg)
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
                Err(ContractError::SameProxyCodeId {})
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("config", "Proxy Code Id")
        .add_attribute("proxy_code_id", format!("{}", updated_code_id)))
}

/// Updates the latest proxy multisig code id for the supported `wallet_proxy`
fn update_proxy_multisig_code_id(
    deps: DepsMut,
    info: MessageInfo,
    new_code_id: u64,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;
    let updated_code_id = PROXY_MULTISIG_CODE_ID.update(
        deps.storage,
        |mut current_code_id| -> Result<_, ContractError> {
            if current_code_id != new_code_id {
                current_code_id = new_code_id;
                Ok(current_code_id)
            } else {
                Err(ContractError::SameProxyMultisigCodeId {})
            }
        },
    )?;

    Ok(Response::new()
        .add_attribute("config", "Proxy Multisig Code Id")
        .add_attribute("proxy_multisig_code_id", format!("{}", updated_code_id)))
}

fn update_wallet_fee(
    deps: DepsMut,
    info: MessageInfo,
    new_fee: u128,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    let fee = Uint128::new(new_fee);
    FEE.save(deps.storage, &fee)?;
    Ok(Response::new()
        .add_attribute("action", "wallet_fee_updated")
        .add_attribute("new fee", fee))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, StdError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert
    let expected_id = TOTAL_CREATED.load(deps.storage)?;
    if reply.id == expected_id {
        let data = reply.result.into_result().map_err(StdError::generic_err)?;
        let first_instantiate_event = data
            .events
            .iter()
            .find(|e| e.ty == "instantiate")
            .ok_or_else(|| StdError::generic_err("Reply: Unable to find reply event"))?;

        // When running in multitest the key for addr is _contract_addr
        // However, it is _contract_address when deployed to wasmd chain
        // TODO: issue
        let str_addr = &first_instantiate_event.attributes[0].value;
        let wallet_addr: CanonicalAddr = deps.api.addr_canonicalize(str_addr)?;
        WALLETS.save(deps.storage, &wallet_addr, &())?;

        let res = Response::new()
            .add_attribute("action", "Wallet Proxy Stored")
            .add_attribute("proxy_address", str_addr);
        Ok(res)
    } else {
        Err(StdError::GenericErr {
            msg: ContractError::InvalidReplyId {}.to_string(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Wallets {} => to_binary(&query_wallet_list(deps)?),
        QueryMsg::ProxyCodeId {} => to_binary(&query_proxy_code_id(deps)?),
        QueryMsg::MultisigCodeId {} => to_binary(&query_multisig_code_id(deps)?),
    }
}

/// Returns all the wallets created
pub fn query_wallet_list(deps: Deps) -> StdResult<WalletListResponse> {
    let wallets: Result<Vec<_>, _> = WALLETS
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key?)))
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
}

/// Returns the current supported `wallet_proxy` code id
pub fn query_proxy_code_id(deps: Deps) -> StdResult<u64> {
    let id = PROXY_CODE_ID.load(deps.storage)?;
    Ok(id)
}

/// Returns the current default `multisig` code id for `wallet_proxy`
/// wallet user can use their own version, however we only support the cw3-fixed-multisig
/// `instantiateMsg` for the time being
pub fn query_multisig_code_id(deps: Deps) -> StdResult<u64> {
    let id = PROXY_MULTISIG_CODE_ID.load(deps.storage)?;
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
