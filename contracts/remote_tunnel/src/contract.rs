#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, QueryResponse,
    Reply, Response, StdResult, Uint128,
};

use cw_utils::parse_reply_instantiate_data;
use vectis_wallet::{
    DispatchResponse, PacketMsg, RemoteTunnelPacketMsg, StdAck, PACKET_LIFETIME,
    RECEIVE_DISPATCH_ID,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Config, CONFIG, DAO, DAO_TUNNEL_CHANNEL, DENOM, FACTORY, IBC_TRANSFER_CHANNEL, RESULTS,
};
use crate::{ContractError, FACTORY_CALLBACK_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let cfg = Config {
        connection_id: msg.connection_id,
        ibc_transfer_port_id: msg.ibc_transfer_port_id,
        dao_tunnel_port_id: msg.dao_tunnel_port_id,
    };
    CONFIG.save(deps.storage, &cfg)?;
    DENOM.save(deps.storage, &msg.denom)?;

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
        ExecuteMsg::DaoActions { msg, job_id } => execute_dispatch(deps, env, info, msg, job_id),
        ExecuteMsg::IbcTransfer { addr } => execute_ibc_transfer(deps, env, info, addr),
    }
}

pub fn execute_mint_govec(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wallet_addr: String,
) -> Result<Response, ContractError> {
    let factory_addr = FACTORY
        .may_load(deps.storage)?
        .ok_or(ContractError::NotFound("Factory".to_string()))?;

    if deps.api.addr_humanize(&factory_addr)? != info.sender {
        return Err(ContractError::Unauthorized);
    }

    let packet = RemoteTunnelPacketMsg::MintGovec {
        wallet_addr: wallet_addr.clone(),
    };

    let channel_id = DAO_TUNNEL_CHANNEL.load(deps.storage)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "mint_govec")
        .add_attribute("wallet_addr", wallet_addr))
}

pub fn execute_dispatch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: RemoteTunnelPacketMsg,
    job_id: Option<String>,
) -> Result<Response, ContractError> {
    let packet = PacketMsg {
        sender: info.sender.to_string(),
        job_id,
        msg: to_binary(&msg)?,
    };

    let channel_id = DAO_TUNNEL_CHANNEL.load(deps.storage)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_dispatch"))
}

pub fn execute_ibc_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to_address: Option<String>,
) -> Result<Response, ContractError> {
    let channel_id = IBC_TRANSFER_CHANNEL.load(deps.storage)?;
    // if to_address is none, it goes to the DAO
    let to_address = to_address.unwrap_or(DAO.load(deps.storage)?);
    if info.funds.is_empty() {
        return Err(ContractError::EmptyFund {});
    }
    let denom = DENOM.load(deps.storage)?;
    let fund_amount = info.funds.iter().try_fold(Uint128::zero(), |acc, c| {
        if c.denom == denom {
            Ok(acc + c.amount)
        } else {
            return Err(ContractError::EmptyFund {});
        }
    })?;

    let msg = IbcMsg::Transfer {
        channel_id,
        to_address: to_address.clone(),
        amount: coin(fund_amount.u128(), denom),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_ibc_transfer")
        .add_attribute("to", to_address))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Factory {} => to_binary(&query_factory(deps)?),
        QueryMsg::Channel {} => to_binary(&query_channel(deps)?),
    }
}

pub fn query_factory(deps: Deps) -> StdResult<Option<Addr>> {
    let addr = match FACTORY.may_load(deps.storage)? {
        Some(c) => Some(deps.api.addr_humanize(&c)?),
        _ => None,
    };
    Ok(addr)
}

pub fn query_channel(deps: Deps) -> StdResult<Option<String>> {
    let channel = DAO_TUNNEL_CHANNEL.may_load(deps.storage)?;
    Ok(channel)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        FACTORY_CALLBACK_ID => reply_inst_callback(deps, reply),
        RECEIVE_DISPATCH_ID => reply_dispatch_callback(deps, reply),
        _ => Err(ContractError::InvalidReplyId),
    }
}

pub fn reply_inst_callback(deps: DepsMut, reply: Reply) -> Result<Response, ContractError> {
    let reply = parse_reply_instantiate_data(reply)?;
    let addr = deps.api.addr_canonicalize(&reply.contract_address)?;

    FACTORY.save(deps.storage, &addr)?;
    Ok(Response::new())
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
