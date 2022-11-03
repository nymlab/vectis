use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order, Reply,
    Response, StdResult, SubMsgResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use vectis_wallet::{
    DaoTunnelPacketMsg, PacketMsg, StdAck, WalletFactoryInstantiateMsg, DEFAULT_LIMIT, MAX_LIMIT,
    PACKET_LIFETIME,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RemoteTunnels};
use crate::state::{ADMIN, GOVEC, IBC_TUNNELS};
use std::convert::Into;

const CONTRACT_NAME: &str = "crates.io:vectis-dao-tunnel";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
        } => execute_add_approved_controller(deps, info, connection_id, port_id, true),
        ExecuteMsg::RemoveApprovedController {
            connection_id,
            port_id,
        } => execute_add_approved_controller(deps, info, connection_id, port_id, false),
        ExecuteMsg::DispatchActionOnRemoteTunnel {
            job_id,
            msg,
            channel_id,
        } => execute_dispatch_to_remote_tunnel(deps, env, info, job_id, msg, channel_id),
    }
}

fn execute_add_approved_controller(
    deps: DepsMut,
    info: MessageInfo,
    connection_id: String,
    port_id: String,
    to_add: bool,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    if to_add {
        IBC_TUNNELS.save(deps.storage, (connection_id.clone(), port_id.clone()), &())?;

        Ok(Response::new()
            .add_attribute("action", "add_approved_controller")
            .add_attribute("connection_id", connection_id)
            .add_attribute("port_id", port_id))
    } else {
        IBC_TUNNELS.remove(deps.storage, (connection_id.clone(), port_id.clone()));

        Ok(Response::new()
            .add_attribute("action", "remove_approved_controller")
            .add_attribute("connection_id", connection_id)
            .add_attribute("port_id", port_id))
    }
}

fn execute_instantiate_remote_factory(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    job_id: u64,
    code_id: u64,
    msg: WalletFactoryInstantiateMsg,
    channel_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let packet = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::InstantiateFactory { code_id, msg })?,
    };

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_instantiate_remote_factory"))
}

fn execute_dispatch_to_remote_tunnel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    job_id: u64,
    msg: DaoTunnelPacketMsg,
    sending_channel_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let packet = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&msg)?,
    };

    let msg = IbcMsg::SendPacket {
        channel_id: sending_channel_id.clone(),
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_remote_tunnel_channel"))
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
    start_after: Option<(String, String)>,
    limit: Option<u32>,
) -> StdResult<RemoteTunnels> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(|s| Bound::exclusive((s.0, s.1)));

    let tunnels: StdResult<Vec<(String, String)>> = IBC_TUNNELS
        .sub_prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|e| -> StdResult<(String, String)> { Ok(e?.0) })
        .collect();

    Ok(RemoteTunnels { tunnels: tunnels? })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        // All possible DAO actions
        // VectisDaoActionIds::GovecMint = 10
        // VectisDaoActionIds::ProposalExecute = 18
        10..=18 => reply_dao_actions(reply),
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn reply_dao_actions(reply: Reply) -> Result<Response, ContractError> {
    let res = Response::new();
    match reply.result {
        SubMsgResult::Ok(_) => Ok(res.set_data(StdAck::success(&reply.id))),
        SubMsgResult::Err(e) => Ok(res.set_data(StdAck::fail(e))),
    }
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
