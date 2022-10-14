#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, QueryResponse, Reply,
    Response, StdResult,
};

use cw_utils::parse_reply_instantiate_data;
use vectis_wallet::{PacketMsg, StdAck, PACKET_LIFETIME, RECEIVE_DISPATCH_ID, DispatchResponse};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CHANNEL, CONFIG, FACTORY, RESULTS};
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
        port_id: msg.port_id,
    };
    CONFIG.save(deps.storage, &cfg)?;

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
        ExecuteMsg::MintGovec { wallet_addr } => execute_mint_govec(deps, env, info, wallet_addr),
        ExecuteMsg::Dispatch { msgs, job_id  } => execute_dispatch(deps, env, info, msgs, job_id),
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

    let packet = PacketMsg::MintGovec {
        wallet_addr: wallet_addr.clone(),
    };

    let channel_id = CHANNEL.load(deps.storage)?;

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
    msgs: Vec<CosmosMsg>,
    job_id: Option<String>
) -> Result<Response, ContractError> {
    let packet = PacketMsg::Dispatch {
        sender: info.sender.to_string(),
        job_id,
        msgs,
    };

    let channel_id = CHANNEL.load(deps.storage)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_dispatch"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Factory => to_binary(&query_factory(deps)?),
        QueryMsg::Channel => to_binary(&query_channel(deps)?),
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
    let channel = CHANNEL.may_load(deps.storage)?;
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
