use cosmwasm_std::{
    entry_point, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Reply,
    Response, StdResult,
};
use cw2::set_contract_version;
use vectis_wallet::{PacketMsg, WalletFactoryInstantiateMsg, PACKET_LIFETIME};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADMIN, IBC_CONTROLLERS};

const CONTRACT_NAME: &str = "crates.io:vectis-ibc-host";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;
    ADMIN.save(deps.storage, &admin_addr)?;
    Ok(Response::new())
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
        ExecuteMsg::Dispatch {
            msgs,
            job_id,
            channel_id,
        } => execute_dispatch(deps, env, info, msgs, job_id, channel_id),
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

    IBC_CONTROLLERS
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

    let packet = PacketMsg::InstantiateFactory { code_id, msg };

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_instantiate_remote_factory"))
}

pub fn execute_dispatch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msgs: Vec<CosmosMsg>,
    job_id: Option<String>,
    channel_id: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;

    let packet = PacketMsg::Dispatch {
        sender: info.sender.to_string(),
        job_id,
        msgs,
    };

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "execute_dispatch"))
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
        data: to_binary(&PacketMsg::UpdateChannel)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "update_remote_tunnel_channel")
        .add_attribute("channel_id", channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _reply: Reply) -> Result<Response, ContractError> {
    Ok(Response::new())
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
