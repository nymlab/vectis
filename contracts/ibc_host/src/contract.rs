use cosmwasm_std::{
    entry_point, from_slice, to_binary, wasm_execute, BankMsg, CosmosMsg, Deps, DepsMut, Empty,
    Env, Event, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcQuery, IbcReceiveResponse, MessageInfo, Order,
    QueryRequest, QueryResponse, Reply, Response, StdResult, SubMsg, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{ADMIN, IBC_CONTROLLERS};
use cw2::set_contract_version;
use vectis_wallet::{check_order, check_version, PacketMsg, StdAck, IBC_APP_VERSION};
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

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddApprovedController {
            connection_id,
            port_id,
        } => execute_add_approved_controller(deps, info, connection_id, port_id),
    }
}

fn execute_add_approved_controller(
    deps: DepsMut,
    info: MessageInfo,
    connection_id: String,
    port_id: String,
) -> Result<Response, ContractError> {
    // check it is DAO
    // add to IBC_CONTROLLERS
    unimplemented!();
}

#[entry_point]
/// enforces ordering and versioing constraints
/// note: anyone can create a channel but only the DAO approved (connection_id, port) will be able
/// to reflect calls
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    let channel = msg.channel();

    check_order(&channel.order)?;
    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
/// note: anyone can create a channel but only the DAO approved (connection_id, port) will be able
/// to reflect calls
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let chan_id = &channel.endpoint.channel_id;
    let port_id = &channel.endpoint.port_id;
    let connection_id = &channel.connection_id;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("port_id", port_id)
        .add_attribute("channel_id", chan_id)
        .add_attribute("connection_id", connection_id)
        .add_event(Event::new("ibc").add_attribute("channel", "connect")))
}

#[entry_point]
/// On closed channel, we take all tokens from reflect contract to this contract.
/// We also delete the channel entry from accounts.
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let chan_id = &channel.endpoint.channel_id;
    let port_id = &channel.endpoint.port_id;
    let connection_id = &channel.connection_id;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("port_id", port_id)
        .add_attribute("channel_id", chan_id)
        .add_attribute("connection_id", connection_id))
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    Ok(Response::new())
}

#[entry_point]
/// we look for a the proper reflect contract to relay to and send the message
/// We cannot return any meaningful response value as we do not know the response value
/// of execution. We just return ok if we dispatched, error if we failed to dispatch
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = msg.packet;
    let caller_channel_id = packet.src.channel_id;
    let caller_port_id = packet.src.port_id;

    is_authorised_controller(deps.as_ref(), caller_channel_id, caller_port_id)?;
    match from_slice(&packet.data)? {
        PacketMsg::Dispatch {
            msgs,
            sender,
            callback_id,
        } => receive_dispatch(deps, msgs, sender, callback_id),
        _ => Err(ContractError::InvalidDispatch {}),
    }
}

/// Makes sure that the incoming connection is from a smart contract that the DAO has approved
fn is_authorised_controller(
    deps: Deps,
    caller_channel_id: String,
    caller_port_id: String,
) -> Result<(), ContractError> {
    let connection_id = deps.querier.query(&QueryRequest::Ibc(IbcQuery::Channel {
        channel_id: caller_channel_id,
        port_id: Some(caller_port_id.clone()),
    }))?;

    if IBC_CONTROLLERS
        .may_load(deps.storage, (connection_id, caller_port_id))?
        .is_some()
    {
        Ok(())
    } else {
        Err(ContractError::InvalidController {})
    }
}

fn receive_dispatch(
    deps: DepsMut,
    msgs: Vec<CosmosMsg>,
    sender: String,
    callback_id: Option<String>,
) -> Result<IbcReceiveResponse, ContractError> {
    Ok(IbcReceiveResponse::new().add_attribute("action", "receive_dispatch"))
}

#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}
