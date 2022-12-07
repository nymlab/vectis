#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response,
    StdError, StdResult,
};
use cw_utils::parse_reply_execute_data;

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::{create_mint_msg, ensure_has_govec, handle_govec_minted};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UnclaimedWalletList};
use crate::state::{GOVEC_CLAIM_LIST, GOVEC_MINTER, TOTAL_CREATED};

use vectis_wallet::CreateWalletMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use vectis_wallet::factory_queries::{
    query_code_id, query_dao_addr, query_fees, query_total, query_unclaim_wallet_list,
    query_wallet_claim_expiration,
};
use vectis_wallet::{
    ensure_is_dao, ensure_is_enough_claim_fee, factory_execute, factory_instantiate,
    handle_proxy_instantion_reply, GOVEC_REPLY_ID,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if let Some(mint) = msg.govec_minter.clone() {
        GOVEC_MINTER.save(deps.storage, &deps.api.addr_canonicalize(&mint)?)?;
    };
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
            create_wallet(deps, info, env, create_wallet_msg)
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

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    ensure_has_govec(deps.as_ref())?;
    factory_execute::create_wallet(deps, info, env, create_wallet_msg)
}

fn update_govec_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender.as_str())?;
    GOVEC_MINTER.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;

    let event = Event::new("vectis.factory.v1.MsgUpdateGovecAddr").add_attribute("address", addr);

    Ok(Response::new().add_event(event))
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
        let mint_msg = create_mint_msg(deps.as_ref(), info.sender.to_string())?;

        let event = Event::new("vectis.factory.v1.MsgClaimGovec")
            .add_attribute("proxy_address", info.sender);

        Ok(Response::new().add_submessage(mint_msg).add_event(event))
    }
}

fn govec_minted(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sucess: bool,
    _wallet: String,
) -> Result<Response, ContractError> {
    Err(ContractError::NotSupportedByChain {})
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
        if reply.id == GOVEC_REPLY_ID {
            let res = parse_reply_execute_data(reply)?;
            if let Some(b) = res.data {
                let addr: String = from_binary(&b)?;
                return handle_govec_minted(deps, addr);
            } else {
                return Err(ContractError::InvalidReplyFromGovec {});
            }
        }

        Err(ContractError::InvalidReplyId {})
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

pub fn query_pending_unclaim_wallet_list(
    _deps: Deps,
    _start_after: Option<String>,
    _limit: Option<u32>,
) -> StdResult<UnclaimedWalletList> {
    Err(StdError::GenericErr {
        msg: String::from("Not supported"),
    })
}

/// Returns govec token address
pub fn query_govec_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&GOVEC_MINTER.load(deps.storage)?)
}
