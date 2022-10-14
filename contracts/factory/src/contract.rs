use crate::error::ContractError;
use crate::helpers::{
    ensure_enough_native_funds, ensure_has_govec, ensure_is_admin, ensure_is_valid_migration_msg,
    ensure_is_valid_threshold,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use crate::state::{
    ADDR_PREFIX, ADMIN, FEE, GOVEC, GOVEC_CLAIM_LIST, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID,
    TOTAL_CREATED,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Expiration, DAY};
use vectis_govec::msg::ExecuteMsg::Mint;
pub use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, CreateWalletMsg, Guardians,
    MigrationMsgError, ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction, RelayTxError,
    WalletAddr, WalletInfo, GOVEC_CLAIM_DURATION_DAY_MUL,
};
// use stake_cw20::msg::InstantiateMsg as StakingInstantiateMsg;
use std::ops::{Add, Mul};
use vectis_proxy::msg::{InstantiateMsg as ProxyInstantiateMsg, QueryMsg as ProxyQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// settings for pagination for wallet list
const MAX_LIMIT: u32 = 1000;
const DEFAULT_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
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
    FEE.save(deps.storage, &msg.wallet_fee)?;
    if let Some(addr) = msg.govec {
        GOVEC.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    }

    Ok(Response::new().add_attribute("Vectis Factory instantiated", env.contract.address))
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
            create_wallet(deps, info, env, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            migration_msg,
        } => migrate_wallet(deps, info, wallet_address, migration_msg),
        ExecuteMsg::UpdateCodeId { ty, new_code_id } => update_code_id(deps, info, ty, new_code_id),
        ExecuteMsg::UpdateWalletFee { new_fee } => update_wallet_fee(deps, info, new_fee),
        ExecuteMsg::UpdateGovecAddr { addr } => update_govec_addr(deps, info, addr),
        ExecuteMsg::UpdateDao { addr } => update_admin_addr(deps, info, addr),
        ExecuteMsg::ClaimGovec {} => claim_govec(deps, env, info),
        ExecuteMsg::PurgeExpiredClaims { start_after, limit } => {
            purge_expired_claims(deps, env, start_after, limit)
        }
    }
}

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    ensure_has_govec(deps.as_ref())?;
    let fee = FEE.load(deps.storage)?;
    let proxy_init_funds = create_wallet_msg.proxy_initial_funds.clone();
    let multisig_initial_funds = create_wallet_msg
        .guardians
        .guardians_multisig
        .clone()
        .unwrap_or_default()
        .multisig_initial_funds;

    // Ensure fixed multisig threshold is valid, if provided
    ensure_is_valid_threshold(&create_wallet_msg.guardians)?;
    ensure_enough_native_funds(
        &fee,
        &proxy_init_funds,
        &multisig_initial_funds,
        &info.funds,
    )?;

    if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(1) {
        // The wasm message containing the `wallet_proxy` instantiation message
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: PROXY_CODE_ID.load(deps.storage)?,
            msg: to_binary(&ProxyInstantiateMsg {
                multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                create_wallet_msg,
                code_id: PROXY_CODE_ID.load(deps.storage)?,
                addr_prefix: ADDR_PREFIX.load(deps.storage)?,
            })?,
            funds: vec![proxy_init_funds, multisig_initial_funds].concat(),
            label: "Wallet-Proxy".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, next_id);
        let res = Response::new().add_submessage(msg);

        // Send native tokens to admin to join the DAO
        TOTAL_CREATED.save(deps.storage, &next_id)?;

        if fee.amount != Uint128::zero() {
            let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&ADMIN.load(deps.storage)?)?
                    .to_string(),
                amount: vec![fee],
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

/// Updates the latest code id for the supported `wallet_proxy`
fn update_code_id(
    deps: DepsMut,
    info: MessageInfo,
    ty: CodeIdType,
    new_code_id: u64,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;
    match ty {
        CodeIdType::Proxy => {
            PROXY_CODE_ID.update(deps.storage, |c| {
                if c == new_code_id {
                    Err(ContractError::SameProxyCodeId {})
                } else {
                    Ok(new_code_id)
                }
            })?;
        }
        CodeIdType::Multisig => {
            PROXY_MULTISIG_CODE_ID.update(deps.storage, |c| {
                if c == new_code_id {
                    Err(ContractError::SameProxyMultisigCodeId {})
                } else {
                    Ok(new_code_id)
                }
            })?;
        }
    }
    Ok(Response::new()
        .add_attribute("config", "Code Id")
        .add_attribute("type", format!("{:?}", ty))
        .add_attribute("new Id", format!("{}", new_code_id)))
}

fn update_wallet_fee(
    deps: DepsMut,
    info: MessageInfo,
    new_fee: Coin,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    FEE.save(deps.storage, &new_fee)?;
    Ok(Response::new()
        .add_attribute("config", "Wallet Fee")
        .add_attribute("New Fee", format!("{}", new_fee)))
}

fn update_govec_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    GOVEC.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "Govec Addr")
        .add_attribute("New Addr", addr))
}

fn update_admin_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    ADMIN.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "Admin")
        .add_attribute("New Admin", addr))
}

fn claim_govec(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let claiming_user = deps.api.addr_canonicalize(info.sender.as_str())?.to_vec();
    if GOVEC_CLAIM_LIST
        .load(deps.storage, claiming_user.clone())?
        .is_expired(&env.block)
    {
        GOVEC_CLAIM_LIST.remove(deps.storage, claiming_user);
        return Err(ContractError::ClaimExpired {});
    } else {
        let mint_msg: SubMsg = SubMsg::new(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&GOVEC.load(deps.storage)?)?
                .to_string(),
            msg: to_binary(&Mint {
                new_wallet: info.sender.to_string(),
            })?,
            funds: vec![],
        });

        let res = Response::new()
            .add_submessage(mint_msg)
            .add_attribute("action", "Claimed Govec")
            .add_attribute("proxy_address", info.sender);
        Ok(res)
    }
}

fn purge_expired_claims(
    deps: DepsMut,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            Some(Bound::exclusive(wallet_addr))
        }
        None => None,
    };

    let wallets: StdResult<Vec<(Vec<u8>, Expiration)>> = GOVEC_CLAIM_LIST
        .prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| -> StdResult<(Vec<u8>, Expiration)> { w })
        .collect();

    for w in wallets? {
        if w.1.is_expired(&env.block) {
            GOVEC_CLAIM_LIST.remove(deps.storage, w.0)
        }
    }

    Ok(Response::default())
}

/// reply hooks handles replies from proxy wallet instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert

    let expected_id = TOTAL_CREATED.load(deps.storage)?;
    if reply.id == expected_id {
        if let Ok(res) = parse_reply_instantiate_data(reply) {
            let wallet_addr: CanonicalAddr = deps.api.addr_canonicalize(&res.contract_address)?;
            let addr_bin = res.data.ok_or(ContractError::ProxyInstantiationError {})?;
            let expiration = Expiration::AtTime(env.block.time)
                .add(DAY.mul(GOVEC_CLAIM_DURATION_DAY_MUL))
                .expect("error defining activate_at");

            GOVEC_CLAIM_LIST.save(deps.storage, wallet_addr.to_vec(), &expiration)?;

            let res = Response::new()
                .add_attribute("action", "Govec claim list updated")
                .add_attribute("proxy_address", res.contract_address);
            Ok(res)
        } else {
            Err(ContractError::ProxyInstantiationError {})
        }
    } else {
        Err(ContractError::InvalidReplyId {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UnclaimedGovecWallets { start_after, limit } => {
            to_binary(&query_unclaim_wallet_list(deps, env, start_after, limit)?)
        }
        QueryMsg::ClaimExpiration { wallet } => {
            to_binary(&query_wallet_claim_expiration(deps, wallet)?)
        }
        QueryMsg::CodeId { ty } => to_binary(&query_code_id(deps, ty)?),
        QueryMsg::Fee {} => to_binary(&query_fee(deps)?),
        QueryMsg::GovecAddr {} => to_binary(&query_govec_addr(deps)?),
        QueryMsg::DaoAddr {} => to_binary(&query_admin_addr(deps)?),
        QueryMsg::TotalCreated {} => to_binary(&query_total(deps)?),
    }
}

/// Returns fees required for wallet creation
pub fn query_fee(deps: Deps) -> StdResult<Coin> {
    FEE.load(deps.storage)
}

/// Returns wallets created with limit
pub fn query_unclaim_wallet_list(
    deps: Deps,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<WalletListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            Some(Bound::exclusive(wallet_addr))
        }
        None => None,
    };
    let wallets: StdResult<Vec<_>> = GOVEC_CLAIM_LIST
        .prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| -> StdResult<Addr> { deps.api.addr_humanize(&CanonicalAddr::from(w?.0)) })
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
}
/// Returns wallets of user
pub fn query_wallet_claim_expiration(deps: Deps, wallet: String) -> StdResult<Expiration> {
    GOVEC_CLAIM_LIST.load(deps.storage, deps.api.addr_canonicalize(&wallet)?.to_vec())
}

/// Returns the current supported code Id:
/// - `wallet_proxy`
///  - `multisig` wallet user can use their own version, however we only support the cw3-fixed-multisig
pub fn query_code_id(deps: Deps, ty: CodeIdType) -> StdResult<u64> {
    let id = match ty {
        CodeIdType::Proxy => PROXY_CODE_ID.load(deps.storage)?,
        CodeIdType::Multisig => PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
    };
    Ok(id)
}
/// Returns govec token address
pub fn query_govec_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&GOVEC.load(deps.storage)?)
}

/// Returns admin address
pub fn query_admin_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&ADMIN.load(deps.storage)?)
}

pub fn query_total(deps: Deps) -> StdResult<u64> {
    TOTAL_CREATED.load(deps.storage)
}
