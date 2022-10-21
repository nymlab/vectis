use cosmwasm_std::{
    entry_point, from_slice, to_binary, CosmosMsg, Deps, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcEndpoint, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcQuery, IbcReceiveResponse, QueryRequest, StdError, StdResult, SubMsg, WasmMsg,
};
use vectis_govec::msg::ExecuteMsg as GovecExecuteMsg;
use vectis_wallet::{
    acknowledge_dispatch, check_order, check_version, DispatchResponse, IbcError, PacketMsg,
    StdAck, IBC_APP_VERSION, RECEIVE_DISPATCH_ID,
};

use crate::state::{GOVEC, IBC_TUNNELS, RESULTS};
use crate::{ContractError, MING_DISPATCH_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
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
    // In ibcv3 we don't check the version string passed in the IbcChannel
    // and only check the counterparty version in OpenTry
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }
    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let original_packet: PacketMsg = from_slice(&msg.original_packet.data)?;
    match original_packet {
        PacketMsg::Dispatch { job_id, sender, .. } => {
            Ok(acknowledge_dispatch(job_id, sender, msg)?)
        }
        _ => Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet = msg.packet;
    is_authorised_src(deps.as_ref(), packet.src, packet.dest)?;
    match from_slice(&packet.data)? {
        PacketMsg::Dispatch { msgs, .. } => receive_dispatch(deps, msgs),
        PacketMsg::MintGovec { wallet_addr } => receive_mint_govec(deps, wallet_addr),
        _ => Err(ContractError::IbcError(IbcError::InvalidPacket)),
    }
}

fn receive_mint_govec(
    deps: DepsMut,
    wallet_addr: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let acknowledgement = StdAck::success(&false);

    let contract_addr = deps.api.addr_humanize(&GOVEC.load(deps.storage)?)?;

    let msg = to_binary(&GovecExecuteMsg::Mint {
        new_wallet: wallet_addr,
    })?;

    let msg = SubMsg::reply_on_success(
        WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg,
            funds: vec![],
        },
        MING_DISPATCH_ID,
    );

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(msg)
        .add_attribute("action", "vectis_dao_tunnel_receive_mint_govec"))
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

#[cfg_attr(not(feature = "library"), entry_point)]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    // We currently do not save the channel_id to call the remote_tunnels
    is_authorised_src(
        deps.as_ref(),
        msg.channel().counterparty_endpoint.clone(),
        msg.channel().endpoint.clone(),
    )
    .map_err(|e| StdError::generic_err(e.to_string()))?;
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", &msg.channel().endpoint.channel_id)
        .add_attribute("src port_id", &msg.channel().counterparty_endpoint.port_id))
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

/// Makes sure that the incoming msg is is from a smart contract that the DAO has approved
/// see IBC_TUNNELS for state details
fn is_authorised_tunnel(
    deps: Deps,
    local_connection_id: String,
    caller_port_id: String,
) -> Result<(), ContractError> {
    if IBC_TUNNELS
        .may_load(deps.storage, (local_connection_id, caller_port_id))?
        .is_none()
    {
        return Err(ContractError::InvalidTunnel {});
    }

    Ok(())
}

/// Wrapper around `is_authorised_tunnel` to query for the correct connection_id of the underlying
/// light client
fn is_authorised_src(
    deps: Deps,
    counterparty_endpoint: IbcEndpoint,
    endpoint: IbcEndpoint,
) -> Result<(), ContractError> {
    let connection_id = deps.querier.query(&QueryRequest::Ibc(IbcQuery::Channel {
        channel_id: endpoint.channel_id,
        port_id: Some(endpoint.port_id.clone()),
    }))?;

    is_authorised_tunnel(deps, connection_id, counterparty_endpoint.port_id)
}
