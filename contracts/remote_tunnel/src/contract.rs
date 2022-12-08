#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Coin, Deps, DepsMut, Env, Event, IbcMsg, MessageInfo, QueryResponse, Reply,
    Response, StdResult, Uint128,
};
use cw_storage_plus::Bound;

use cw_utils::parse_reply_instantiate_data;
use vectis_wallet::{
    DaoConfig, PacketMsg, RemoteTunnelPacketMsg, StdAck, VectisDaoActionIds, DEFAULT_LIMIT,
    MAX_LIMIT, PACKET_LIFETIME,
};

use crate::msg::{
    ChainConfigResponse, ExecuteMsg, IbcTransferChannels, InstantiateMsg, QueryMsg, Receiver,
};
use crate::state::{CHAIN_CONFIG, DAO_CONFIG, IBC_TRANSFER_MODULES, JOB_ID};
use crate::{ContractError, DISPATCH_CALLBACK_ID, FACTORY_CALLBACK_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    DAO_CONFIG.save(deps.storage, &msg.dao_config)?;
    CHAIN_CONFIG.save(deps.storage, &msg.chain_config)?;
    if let Some(init_ibc_transfer_mods) = msg.init_ibc_transfer_mod {
        for module in init_ibc_transfer_mods.endpoints {
            // Ignore the unestablished channel_id
            IBC_TRANSFER_MODULES.save(deps.storage, module.0, &module.1)?;
        }
    }

    Ok(Response::new().add_attribute("vectis-remote-tunnel instantiated", env.contract.address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DaoActions { msg } => execute_dispatch(deps, env, info, msg),
        ExecuteMsg::IbcTransfer { receiver } => execute_ibc_transfer(deps, env, info, receiver),
    }
}

pub fn execute_mint_govec(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wallet_addr: String,
) -> Result<Response, ContractError> {
    let factory_addr = CHAIN_CONFIG
        .load(deps.storage)?
        .remote_factory
        .ok_or(ContractError::FactoryNotAvailable)?;

    if deps.api.addr_humanize(&factory_addr)? != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let mint_govec_msg = RemoteTunnelPacketMsg::MintGovec {
        wallet_addr: wallet_addr.clone(),
    };

    let job_id = JOB_ID.load(deps.storage).unwrap_or(0);
    let packet = PacketMsg {
        sender: info.sender.to_string(),
        job_id,
        msg: to_binary(&mint_govec_msg)?,
    };

    let channel_id = DAO_CONFIG
        .load(deps.storage)?
        .dao_tunnel_channel
        .ok_or(ContractError::DaoChannelNotFound)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };
    JOB_ID.save(deps.storage, &job_id.wrapping_add(1))?;

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "mint_govec requested")
        .add_attribute("wallet_addr", wallet_addr))
}

pub fn execute_dispatch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: RemoteTunnelPacketMsg,
) -> Result<Response, ContractError> {
    // Only the Factory can call this
    if let RemoteTunnelPacketMsg::MintGovec { wallet_addr } = msg {
        execute_mint_govec(deps, env, info, wallet_addr)
    } else {
        let job_id = JOB_ID.load(deps.storage).unwrap_or(0);
        let packet = PacketMsg {
            sender: info.sender.to_string(),
            job_id,
            msg: to_binary(&msg)?,
        };
        let channel_id = DAO_CONFIG
            .load(deps.storage)?
            .dao_tunnel_channel
            .ok_or(ContractError::DaoChannelNotFound)?;

        let msg = IbcMsg::SendPacket {
            channel_id,
            data: to_binary(&packet)?,
            timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
        };

        JOB_ID.save(deps.storage, &job_id.wrapping_add(1))?;

        let event = Event::new("vectis.remote-tunnel.v1.MsgDispatch")
            .add_attribute("job_id", job_id.to_string());

        Ok(Response::new().add_message(msg).add_event(event))
    }
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
    let denom = CHAIN_CONFIG.load(deps.storage)?.denom;
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

    let event = Event::new("vectis.ibc-transfer.v1.MsgIbcTransfer")
        .add_attribute("channel_id", channel_id)
        .add_attribute("to", rcv.addr)
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom.to_string());

    Ok(Response::new().add_message(msg).add_event(event))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::DaoConfig {} => to_binary(&query_dao_config(deps)?),
        QueryMsg::ChainConfig {} => to_binary(&query_chain_config(deps)?),
        QueryMsg::IbcTransferChannels { start_after, limit } => {
            to_binary(&query_channels(deps, start_after, limit)?)
        }
        QueryMsg::NextJobId {} => to_binary(&query_job_id(deps)?),
    }
}

pub fn query_dao_config(deps: Deps) -> StdResult<DaoConfig> {
    DAO_CONFIG.load(deps.storage)
}

pub fn query_chain_config(deps: Deps) -> StdResult<ChainConfigResponse> {
    let config = CHAIN_CONFIG.load(deps.storage)?;
    Ok(ChainConfigResponse {
        denom: config.denom,
        remote_factory: config
            .remote_factory
            .and_then(|addr| deps.api.addr_humanize(&addr).ok()),
    })
}

pub fn query_job_id(deps: Deps) -> StdResult<u64> {
    Ok(JOB_ID.load(deps.storage).unwrap_or(0))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        FACTORY_CALLBACK_ID => reply_inst_callback(deps, reply),
        DISPATCH_CALLBACK_ID => reply_dispatch_callback(reply),
        _ => Err(ContractError::InvalidReplyId),
    }
}

pub fn reply_inst_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    match parse_reply_instantiate_data(reply) {
        Ok(reply) => {
            let addr = deps.api.addr_canonicalize(&reply.contract_address)?;
            CHAIN_CONFIG.update(deps.storage, |mut c| -> StdResult<_> {
                c.remote_factory = Some(addr);
                Ok(c)
            })?;
            Ok(Response::new().set_data(StdAck::success(
                VectisDaoActionIds::FactoryInstantiated as u64,
            )))
        }
        Err(e) => Ok(Response::new().set_data(StdAck::fail(e.to_string()))),
    }
}
/// Callback function for receive_dispatch_actions
/// submessages only reply on error
pub fn reply_dispatch_callback(reply: Reply) -> Result<Response, ContractError> {
    let err = reply.result.unwrap_err();
    Ok(Response::new().set_data(StdAck::fail(err)))
}
