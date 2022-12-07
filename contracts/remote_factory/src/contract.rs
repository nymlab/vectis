#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::helpers::{create_mint_msg, handle_govec_mint_failed, handle_govec_minted};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{DAO, GOVEC_CLAIM_LIST, PENDING_CLAIM_LIST, TOTAL_CREATED};
use cosmwasm_std::{
    to_binary, Addr, Binary, CanonicalAddr, Deps, DepsMut, Env, MessageInfo, Order, Reply,
    Response, StdError, StdResult,
};
use cw2::set_contract_version;

pub use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, CreateWalletMsg, FeeType, FeesResponse,
    Guardians, MigrationMsgError, ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction,
    RelayTxError, WalletAddr, WalletInfo, GOVEC_CLAIM_DURATION_DAY_MUL,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// settings for pagination for unclaimed govec wallet list
const MAX_LIMIT: u32 = 1000;
const DEFAULT_LIMIT: u32 = 50;

use vectis_wallet::factory_queries::{
    query_code_id, query_dao_addr, query_fees, query_total, query_unclaim_wallet_list,
    query_wallet_claim_expiration,
};
use vectis_wallet::{
    ensure_is_enough_claim_fee, factory_execute, factory_instantiate, handle_proxy_instantion_reply,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    factory_instantiate(deps, env, info, msg)
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
            factory_execute::create_wallet(deps, info, env, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            migration_msg,
        } => factory_execute::migrate_wallet(deps, info, wallet_address, migration_msg),
        ExecuteMsg::UpdateCodeId { ty, new_code_id } => {
            factory_execute::update_code_id(deps, info, ty, new_code_id)
        }
        ExecuteMsg::UpdateConfigFee { ty, new_fee } => {
            factory_execute::update_config_fee(deps, info, ty, new_fee)
        }
        ExecuteMsg::UpdateGovecAddr { addr } => update_govec_addr(deps, info, addr),
        ExecuteMsg::UpdateDao { addr } => factory_execute::update_dao_addr(deps, info, addr),
        ExecuteMsg::ClaimGovec {} => claim_govec_or_remove_from_list(deps, env, info),
        ExecuteMsg::GovecMinted {
            success,
            wallet_addr,
        } => govec_minted(deps, env, info, success, wallet_addr),
        ExecuteMsg::PurgeExpiredClaims { start_after, limit } => {
            factory_execute::purge_expired_claims(deps, env, start_after, limit)
        }
    }
}

fn update_govec_addr(
    _deps: DepsMut,
    _info: MessageInfo,
    _addr: String,
) -> Result<Response, ContractError> {
    Err(ContractError::NotSupportedByChain {})
}

fn claim_govec_or_remove_from_list(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let claiming_controller = deps.api.addr_canonicalize(info.sender.as_str())?.to_vec();
    if GOVEC_CLAIM_LIST
        .load(deps.storage, claiming_controller.clone())?
        .is_expired(&env.block)
    {
        GOVEC_CLAIM_LIST.remove(deps.storage, claiming_controller);
        Err(ContractError::ClaimExpired {})
    } else {
        ensure_is_enough_claim_fee(deps.as_ref(), &info.funds)?;

        GOVEC_CLAIM_LIST.remove(deps.storage, claiming_controller.clone());
        PENDING_CLAIM_LIST.save(deps.storage, claiming_controller, &())?;

        let mint_msg = create_mint_msg(deps.as_ref(), info.sender.to_string())?;
        let res = Response::new()
            .add_submessage(mint_msg)
            .add_attribute("action", "Claim Govec Requested")
            .add_attribute("proxy_address", info.sender);
        Ok(res)
    }
}

fn govec_minted(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    success: bool,
    wallet: String,
) -> Result<Response, ContractError> {
    let dao_minter = DAO.load(deps.storage)?;
    if info.sender != deps.api.addr_humanize(&dao_minter)? {
        Err(ContractError::Unauthorized {})
    } else if success {
        handle_govec_minted(deps, wallet)
    } else {
        handle_govec_mint_failed(deps, env, wallet)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UnclaimedGovecWallets { start_after, limit } => {
            to_binary(&query_unclaim_wallet_list(deps, start_after, limit)?)
        }
        QueryMsg::PendingGovecClaimWallets { start_after, limit } => to_binary(
            &query_pending_unclaim_wallet_list(deps, start_after, limit)?,
        ),
        QueryMsg::ClaimExpiration { wallet } => {
            to_binary(&query_wallet_claim_expiration(deps, wallet)?)
        }
        QueryMsg::CodeId { ty } => to_binary(&query_code_id(deps, ty)?),
        QueryMsg::Fees {} => to_binary(&query_fees(deps)?),
        QueryMsg::GovecAddr {} => to_binary(&query_govec_addr(deps)?),
        QueryMsg::DaoAddr {} => to_binary(&query_dao_addr(deps)?),
        QueryMsg::TotalCreated {} => to_binary(&query_total(deps)?),
    }
}

/// Returns wallets on remote waiting for ibc ack
pub fn query_pending_unclaim_wallet_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            Some(Bound::exclusive(wallet_addr))
        }
        None => None,
    };
    let wallets: StdResult<Vec<Addr>> = PENDING_CLAIM_LIST
        .prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| -> StdResult<Addr> {
            let ww = w?;
            deps.api.addr_humanize(&CanonicalAddr::from(ww.0))
        })
        .collect();

    wallets
}

pub fn query_govec_addr(_deps: Deps) -> StdResult<Addr> {
    Err(StdError::GenericErr {
        msg: String::from("Not supported"),
    })
}

/// reply hooks handles replies from proxy wallet instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert

    let expected_id = TOTAL_CREATED.load(deps.storage)?;

    if reply.id == expected_id {
        handle_proxy_instantion_reply(deps, env, reply)
    } else {
        Err(ContractError::InvalidReplyId {})
    }
}
