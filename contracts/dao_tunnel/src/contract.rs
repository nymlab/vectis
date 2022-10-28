use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo,
    Order, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use vectis_wallet::{
    DaoTunnelPacketMsg, DispatchResponse, StdAck, WalletFactoryInstantiateMsg, PACKET_LIFETIME,
    RECEIVE_DISPATCH_ID,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RemoteTunnels};
use crate::state::{ADMIN, GOVEC, IBC_TUNNELS, RESULTS};
use crate::MINT_DISPATCH_ID;

const CONTRACT_NAME: &str = "crates.io:vectis-dao-tunnel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;
    let govec_addr = deps
        .api
        .addr_canonicalize(deps.api.addr_validate(&msg.govec_minter)?.as_str())?;
    GOVEC.save(deps.storage, &govec_addr)?;
    ADMIN.save(deps.storage, &admin_addr)?;
    Ok(Response::new().add_attribute("Vectis DAO-Tunnel instantiated", env.contract.address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddApprovedController {
            connection_id,
            port_id,
        } => execute_add_approved_controller(deps, info, connection_id, port_id),
        ExecuteMsg::InstantiateRemoteFactory {
            code_id,
            msg,
            channel_id,
        } => execute_instantiate_remote_factory(deps, env, info, code_id, msg, channel_id),
        ExecuteMsg::UpdateRemoteTunnelChannel { channel_id } => {
            execute_update_remote_tunnel_channel(deps, env, info, channel_id)
        }
    }
}

fn execute_add_approved_controller(
    deps: DepsMut,
    info: MessageInfo,
    connection_id: String,
    port_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    IBC_TUNNELS
        .save(deps.storage, (connection_id.clone(), port_id.clone()), &())
        .unwrap();

    Ok(Response::new()
        .add_attribute("action", "add_approved_controller")
        .add_attribute("connection_id", connection_id)
        .add_attribute("port_id", port_id))
}

fn execute_instantiate_remote_factory(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    code_id: u64,
    msg: WalletFactoryInstantiateMsg,
    channel_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let packet = DaoTunnelPacketMsg::InstantiateFactory { code_id, msg };

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_instantiate_remote_factory"))
}

fn execute_update_remote_tunnel_channel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    channel_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let msg = IbcMsg::SendPacket {
        channel_id: channel_id.clone(),
        data: to_binary(&DaoTunnelPacketMsg::UpdateChannel)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_remote_tunnel_channel")
        .add_attribute("channel_id", channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Govec {} => to_binary(&query_govec(deps)?),
        QueryMsg::Controllers { start_after, limit } => {
            to_binary(&query_controllers(deps, start_after, limit)?)
        }
    }
}

pub fn query_govec(deps: Deps) -> StdResult<Option<Addr>> {
    let addr = match GOVEC.may_load(deps.storage)? {
        Some(c) => Some(deps.api.addr_humanize(&c)?),
        _ => None,
    };
    Ok(addr)
}

pub fn query_controllers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<RemoteTunnels> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));

    let tunnels: StdResult<Vec<(String, String)>> = IBC_TUNNELS
        .keys(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    Ok(RemoteTunnels { tunnels: tunnels? })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        RECEIVE_DISPATCH_ID => reply_dispatch_callback(deps, reply),
        MINT_DISPATCH_ID => reply_mint_govec(),
        _ => Err(ContractError::InvalidReplyId),
    }
}

pub fn reply_dispatch_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    // add the new result to the current tracker
    let mut results = RESULTS.load(deps.storage)?;
    results.push(reply.result.unwrap().data.unwrap_or_default());
    RESULTS.save(deps.storage, &results)?;

    // update result data if this is the last
    let data = StdAck::success(&DispatchResponse { results });
    Ok(Response::new().set_data(data))
}

pub fn reply_mint_govec() -> Result<Response, ContractError> {
    Ok(Response::new().set_data(StdAck::success(true)))
}

/// Ensures provided addr is the state stored ADMIN
pub fn ensure_is_admin(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller != admin {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}
