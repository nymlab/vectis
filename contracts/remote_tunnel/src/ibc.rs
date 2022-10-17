#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_slice, to_binary, Binary, CosmosMsg, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdResult, SubMsg, WasmMsg,
};

use vectis_wallet::{
    check_connection, check_order, check_port, check_version, DispatchResponse, PacketMsg,
    ReceiveIbcResponseMsg, StdAck, WalletFactoryInstantiateMsg as FactoryInstantiateMsg,
    IBC_APP_VERSION, RECEIVE_DISPATCH_ID,
};

use crate::state::{CHANNEL, CONFIG, FACTORY, RESULTS};
use crate::{ContractError, FACTORY_CALLBACK_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering, versioning and connection constraints
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    let channel = msg.channel();
    let config = CONFIG.load(deps.storage)?;
    check_order(&channel.order)?;
    check_version(&channel.version)?;
    check_connection(&config.connection_id, &channel.connection_id)?;
    check_port(&config.port_id, &channel.endpoint.port_id)?;

    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to parse the ack based on our request
    let original_packet: PacketMsg = from_slice(&msg.original_packet.data)?;
    match original_packet {
        PacketMsg::Dispatch { job_id, sender, .. } => {
            acknowledge_dispatch(deps, env, job_id, sender, msg)
        }
        _ => Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack")),
    }
}

fn acknowledge_dispatch(
    _deps: DepsMut,
    _env: Env,
    job_id: Option<String>,
    sender: String,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res = IbcBasicResponse::new().add_attribute("action", "acknowledge_dispatch");
    match job_id {
        Some(id) => {
            let msg: StdAck = from_slice(&ack.acknowledgement.data)?;
            // Send IBC packet ack message to another contract
            let res = res
                .add_attribute("job_id", &id)
                .add_message(ReceiveIbcResponseMsg { id: id, msg }.into_cosmos_msg(sender)?);
            Ok(res)
        }
        None => Ok(res),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet_msg: PacketMsg = from_slice(&msg.packet.data)?;
    let channel_id = msg.packet.dest.channel_id;
    match packet_msg {
        PacketMsg::UpdateChannel => receive_update_channel(deps, channel_id),
        PacketMsg::Dispatch { msgs, .. } => receive_dispatch(deps, msgs),
        PacketMsg::InstantiateFactory { code_id, msg, .. } => {
            receive_instantiate(deps, code_id, msg)
        }
        PacketMsg::MintGovec { wallet_addr } => receive_mint_govec(deps, wallet_addr),
    }
}

pub fn receive_update_channel(
    deps: DepsMut,
    channel_id: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let acknowledgement = StdAck::success(&());

    CHANNEL.save(deps.storage, &channel_id)?;

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_update_channel"))
}

pub fn receive_dispatch(
    deps: DepsMut,
    msgs: Vec<CosmosMsg>,
) -> Result<IbcReceiveResponse, ContractError> {
    let response = DispatchResponse { results: vec![] };
    let acknowledgement = StdAck::success(&response);

    let sub_msgs: Vec<SubMsg> = msgs
        .iter()
        .map(|m| SubMsg::reply_on_success(m.clone(), RECEIVE_DISPATCH_ID))
        .collect();

    // reset the data field
    RESULTS.save(deps.storage, &vec![])?;

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessages(sub_msgs)
        .add_attribute("action", "vectis_tunnel_remote_receive_dispatch"))
}

pub fn receive_instantiate(
    _deps: DepsMut,
    code_id: u64,
    msg: FactoryInstantiateMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let acknowledgement = StdAck::success(&());

    let msg = WasmMsg::Instantiate {
        admin: None,
        label: "vectis-remote-factory".to_string(),
        code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
    };
    let msg = SubMsg::reply_on_success(msg, FACTORY_CALLBACK_ID);

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(msg)
        .add_attribute("action", "receive_instantiate"))
}

pub fn receive_mint_govec(
    deps: DepsMut,
    _wallet_addr: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let acknowledgement = StdAck::success(&());

    let factory_addr = FACTORY.load(deps.storage)?;

    // TODO: We need the factory msg to let it know we already mint the govec token.
    let msg = WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&factory_addr)?.to_string(),
        msg: Binary::from(vec![]),
        funds: vec![],
    };

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_message(msg)
        .add_attribute("action", "receive_mint_govec"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// we just ignore these temporally. shall we store some info?
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", &msg.channel().endpoint.channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// We don't do anything when a channel is closed
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_close")
        .add_attribute("channel_id", &msg.channel().endpoint.channel_id))
}
