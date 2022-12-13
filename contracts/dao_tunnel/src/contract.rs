use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, Event, IbcMsg, MessageInfo,
    Order, Reply, Response, StdResult, SubMsgResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use vectis_wallet::{
    DaoTunnelPacketMsg, IbcTransferChannels, PacketMsg, Receiver, StdAck, DEFAULT_LIMIT, MAX_LIMIT,
    PACKET_LIFETIME,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RemoteTunnels};
use crate::state::{ADMIN, DENOM, GOVEC, IBC_TRANSFER_MODULES, IBC_TUNNELS};
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
    DENOM.save(deps.storage, &msg.denom)?;
    ADMIN.save(deps.storage, &admin_addr)?;
    if let Some(init_tunnels) = msg.init_remote_tunnels {
        for tunnel in init_tunnels.tunnels {
            IBC_TUNNELS.save(deps.storage, tunnel, &())?;
        }
    }
    if let Some(init_ibc_mods) = msg.init_ibc_transfer_mods {
        for module in init_ibc_mods.endpoints {
            IBC_TRANSFER_MODULES.save(deps.storage, module.0, &module.1)?;
        }
    }

    let event = Event::new("vectis.dao_tunnel.v1.MsgInstantiate")
        .add_attribute("contract_address", env.contract.address);

    Ok(Response::new().add_event(event))
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
        ExecuteMsg::UpdateDaoAddr { new_addr } => execute_update_dao(deps, info, new_addr),
        ExecuteMsg::UpdateGovecAddr { new_addr } => execute_update_govec(deps, info, new_addr),
        ExecuteMsg::UpdateIbcTransferRecieverChannel {
            connection_id,
            channel_id,
        } => execute_update_ibc_transfer_channel(deps, info, connection_id, channel_id),
        ExecuteMsg::IbcTransfer { receiver } => execute_ibc_transfer(deps, env, info, receiver),
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

    let mut event = Event::new("")
        .add_attribute("connection_id", &connection_id)
        .add_attribute("port_id", &port_id);

    if to_add {
        IBC_TUNNELS.save(deps.storage, (connection_id, port_id), &())?;
        event.ty = "vectis.dao_tunnel.v1.MsgAddApprovedController".to_string();
    } else {
        IBC_TUNNELS.remove(deps.storage, (connection_id, port_id));
        event.ty = "vectis.dao_tunnel.v1.MsgRemoveApprovedController".to_string();
    }

    Ok(Response::new().add_event(event))
}
fn execute_update_dao(
    deps: DepsMut,
    info: MessageInfo,
    new_addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    let addr = deps.api.addr_validate(&new_addr)?;
    ADMIN.save(deps.storage, &deps.api.addr_canonicalize(addr.as_str())?)?;

    let event = Event::new("vectis.dao_tunnel.v1.MsgUpdateDaoAddr").add_attribute("address", addr);

    Ok(Response::new().add_event(event))
}

fn execute_update_govec(
    deps: DepsMut,
    info: MessageInfo,
    new_addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    let addr = deps.api.addr_validate(&new_addr)?;
    GOVEC.save(deps.storage, &deps.api.addr_canonicalize(addr.as_str())?)?;

    let event =
        Event::new("vectis.dao_tunnel.v1.MsgUpdateGovecAddr").add_attribute("address", addr);

    Ok(Response::new().add_event(event))
}

fn execute_update_ibc_transfer_channel(
    deps: DepsMut,
    info: MessageInfo,
    connection_id: String,
    channel: Option<String>,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let mut event = Event::new("vectis.dao_tunnel.v1.MsgUpdateIbcTransferRecieverChannel")
        .add_attribute("connection_id", &connection_id);

    match channel {
        Some(c) => {
            // Update the channel
            IBC_TRANSFER_MODULES.save(deps.storage, connection_id, &c)?;
            event = event
                .add_attribute("action", "update")
                .add_attribute("channel_id", &c)
        }
        None => {
            // Remove it
            IBC_TRANSFER_MODULES.remove(deps.storage, connection_id);
            event = event.add_attribute("action", "remove")
        }
    }

    Ok(Response::new().add_event(event))
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

    let event = Event::new("vectis.dao_tunnel.v1.MsgDispatchActionOnRemoteTunnel")
        .add_attribute("channel_id", sending_channel_id)
        .add_attribute("job_id", job_id.to_string());

    Ok(Response::new().add_message(msg).add_event(event))
}

pub fn execute_ibc_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv: Receiver,
) -> Result<Response, ContractError> {
    if info.funds.is_empty() {
        return Err(ContractError::EmptyFund);
    }
    let denom = DENOM.load(deps.storage)?;
    let amount = info.funds.iter().fold(Uint128::zero(), |acc, c| {
        if c.denom == denom {
            acc + c.amount
        } else {
            acc
        }
    });
    if amount.is_zero() {
        return Err(ContractError::EmptyFund);
    };

    let channel_id = IBC_TRANSFER_MODULES
        .load(deps.storage, rcv.connection_id.clone())
        .map_err(|_| ContractError::ChannelNotFound(rcv.connection_id))?;

    // only one type of coin supported in IBC transfer
    let msg = IbcMsg::Transfer {
        channel_id: channel_id.clone(),
        to_address: rcv.addr.clone(),
        amount: Coin {
            denom: denom.clone(),
            amount,
        },
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let event = Event::new("vectis.dao_tunnel.v1.MsgIbcTransfer")
        .add_attribute("to", rcv.addr)
        .add_attribute("channel_id", channel_id)
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom);

    Ok(Response::new().add_message(msg).add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Govec {} => to_binary(&query_govec(deps)?),
        QueryMsg::Dao {} => to_binary(&query_dao(deps)?),
        QueryMsg::Controllers { start_after, limit } => {
            to_binary(&query_controllers(deps, start_after, limit)?)
        }
        QueryMsg::IbcTransferChannels { start_after, limit } => {
            to_binary(&query_channels(deps, start_after, limit)?)
        }
    }
}

pub fn query_channels(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<IbcTransferChannels> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let endpoints: StdResult<Vec<_>> = IBC_TRANSFER_MODULES
        .prefix(())
        .range(deps.storage, start, None, cosmwasm_std::Order::Descending)
        .take(limit)
        .map(|m| -> StdResult<_> {
            let ele = m?;
            Ok((ele.0, ele.1))
        })
        .collect();

    Ok(IbcTransferChannels {
        endpoints: endpoints?,
    })
}

pub fn query_govec(deps: Deps) -> StdResult<Option<Addr>> {
    let addr = match GOVEC.may_load(deps.storage)? {
        Some(c) => Some(deps.api.addr_humanize(&c)?),
        _ => None,
    };
    Ok(addr)
}

pub fn query_dao(deps: Deps) -> StdResult<Option<Addr>> {
    let addr = match ADMIN.may_load(deps.storage)? {
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
        // VectisDaoActionIds::FactoryInstantiated = 20
        10..=20 => reply_dao_actions(reply),
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

pub fn reply_dao_actions(reply: Reply) -> Result<Response, ContractError> {
    let res = Response::new();
    match reply.result {
        SubMsgResult::Ok(_) => Ok(res.set_data(StdAck::success(reply.id))),
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
