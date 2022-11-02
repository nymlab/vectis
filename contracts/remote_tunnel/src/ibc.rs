#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Deps, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdError, StdResult, SubMsg,
    WasmMsg,
};

use vectis_wallet::{
    check_order, check_version, DaoTunnelPacketMsg, IbcError, PacketMsg, RemoteTunnelPacketMsg,
    StdAck, VectisDaoActionIds, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, IBC_APP_VERSION,
};

use crate::state::{CONFIG, DAO_TUNNEL_CHANNEL, IBC_TRANSFER_CHANNEL};
use crate::{ContractError, FACTORY_CALLBACK_ID};

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering, versioning and connection constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    // We dont enforce anything regarding permission here as counterparty_endpoint is not set yet
    let channel = msg.channel();
    check_order(&channel.order)?;
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();

    if ensure_is_dao_tunnel(
        deps.as_ref(),
        &channel.connection_id,
        &channel.counterparty_endpoint.port_id,
    )
    .is_ok()
    {
        if DAO_TUNNEL_CHANNEL.load(deps.storage).is_err() {
            // We only save a new channel if it was not previously set
            DAO_TUNNEL_CHANNEL.save(deps.storage, &channel.endpoint.channel_id)?;
            Ok(IbcBasicResponse::new()
                .add_attribute("action", "ibc_connect")
                .add_attribute(
                    "SAVED dao_tunnel_channel_id",
                    &msg.channel().endpoint.channel_id,
                )
                .add_attribute("dao_tunnel_port_id", &channel.counterparty_endpoint.port_id))
        } else {
            // We accept new channel creation but this will only be used if the DAO calls
            // `UpdateChannel` to update the official channel used to communicate with the
            // dao-tunnel
            Ok(IbcBasicResponse::new()
                .add_attribute("action", "ibc_connect")
                .add_attribute(
                    "IGNORED dao_tunnel_channel_id",
                    &msg.channel().endpoint.channel_id,
                )
                .add_attribute("dao_tunnel_port_id", &channel.counterparty_endpoint.port_id))
        }
    } else if ensure_is_ibc_trasnfer(
        deps.as_ref(),
        &channel.connection_id,
        &channel.counterparty_endpoint.port_id,
    )
    .is_ok()
    {
        // As long as it is the port of the remote ibc transfer module,
        // we can save it as we are not expecting messages to come from this
        IBC_TRANSFER_CHANNEL.save(deps.storage, &channel.endpoint.channel_id)?;
        Ok(IbcBasicResponse::new()
            .add_attribute("action", "ibc_connect")
            .add_attribute(
                "SAVED ibc_transfer_channel_id",
                &msg.channel().endpoint.channel_id,
            )
            .add_attribute(
                "ibc_transfer_port_id",
                &channel.counterparty_endpoint.port_id,
            ))
    } else {
        Err(StdError::generic_err(IbcError::InvalidSrc.to_string()))
    }
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
        .add_attribute("channel_id", &msg.channel().endpoint.channel_id)
        .add_attribute("src_port_id", &msg.channel().counterparty_endpoint.port_id)
        .add_attribute("connection_id", &msg.channel().connection_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    (|| {
        let packet_msg: PacketMsg =
            from_slice(&msg.packet.data).map_err(|_| IbcError::InvalidPacketMsg)?;
        let dao_tunnel_msg: DaoTunnelPacketMsg =
            from_binary(&packet_msg.msg).map_err(|_| IbcError::InvalidInnerMsg)?;
        let dao_channel = DAO_TUNNEL_CHANNEL.load(deps.storage)?;

        // We only need to check for dao_channel here because messages can only be received from
        // authorised / opened channels, which for a remote_tunnel,
        // only connects to the dao_tunnel_channel
        if msg.packet.dest.channel_id == dao_channel {
            match dao_tunnel_msg {
                DaoTunnelPacketMsg::UpdateChannel => {
                    receive_update_channel(deps, msg.packet.dest.channel_id, packet_msg.job_id)
                }
                DaoTunnelPacketMsg::InstantiateFactory { code_id, msg, .. } => {
                    receive_instantiate(deps, code_id, msg, packet_msg.job_id)
                }
            }
        } else {
            Err(ContractError::Unauthorized {})
        }
    })()
    .or_else(|e| {
        Ok(IbcReceiveResponse::new().set_ack(StdAck::fail(format!("IBC Packet Error: {}", e))))
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let res = IbcBasicResponse::new();
    let original_packet_data: PacketMsg = from_binary(&msg.original_packet.data)?;

    let ack_result: StdAck = from_binary(&msg.acknowledgement.data)?;
    // TODO - stdAck(VectisDaoActionIds)
    // THE BELOW IS WRONG
    if original_packet_data.job_id != VectisDaoActionIds::GovecMint as u64 {
        let success = match ack_result {
            StdAck::Result(id) => {
                let reply_id: u64 = from_binary(&id)?;
                // id maps to VectisDaoActionIds
                format!("Success: {}", reply_id)
            }
            StdAck::Error(e) => e,
        };
        Ok(res
            .add_attribute("job_id", original_packet_data.job_id.to_string())
            .add_attribute("result", success))
    } else {
        if let RemoteTunnelPacketMsg::MintGovec { wallet_addr } =
            from_binary(&original_packet_data.msg)?
        {
            let success = match ack_result {
                StdAck::Result(_) => true,
                StdAck::Error(_) => false,
            };
            let submsg = SubMsg::new(WasmMsg::Execute {
                contract_addr: original_packet_data.sender,
                msg: to_binary(&WalletFactoryExecuteMsg::GovecMinted {
                    success,
                    wallet_addr,
                })?,
                funds: vec![],
            });

            Ok(res
                .add_attribute("action", "Mint Govec Ack")
                .add_submessage(submsg))
        } else {
            Err(IbcError::InvalidInnerMsg.into())
        }
    }
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

// Recieve handlers

pub fn receive_update_channel(
    deps: DepsMut,
    channel_id: String,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    DAO_TUNNEL_CHANNEL.save(deps.storage, &channel_id)?;
    Ok(IbcReceiveResponse::new()
        .set_ack(StdAck::success(job_id))
        .add_attribute("action", "dao channel updated")
        .add_attribute("channel", channel_id))
}

pub fn receive_instantiate(
    _deps: DepsMut,
    code_id: u64,
    msg: FactoryInstantiateMsg,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    let msg = WasmMsg::Instantiate {
        admin: None,
        label: "vectis-remote-factory".to_string(),
        code_id,
        msg: to_binary(&msg)?,
        funds: vec![],
    };
    let msg = SubMsg::reply_always(msg, FACTORY_CALLBACK_ID);

    // Set ack in reply
    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "remote factory instantiation")
        .add_attribute("job_id", job_id.to_string()))
}

// utils

fn ensure_is_dao_tunnel(
    deps: Deps,
    local_connection_id: &str,
    src_port_id: &str,
) -> Result<(), ContractError> {
    let dao_tunnel_config = CONFIG.load(deps.storage)?;
    if dao_tunnel_config.connection_id != local_connection_id {
        return Err(IbcError::InvalidConnectionId(dao_tunnel_config.connection_id).into());
    }
    if dao_tunnel_config.dao_tunnel_port_id != src_port_id {
        return Err(IbcError::InvalidPortId(dao_tunnel_config.dao_tunnel_port_id).into());
    }
    Ok(())
}

fn ensure_is_ibc_trasnfer(
    deps: Deps,
    local_connection_id: &str,
    src_port_id: &str,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.connection_id != local_connection_id {
        return Err(IbcError::InvalidConnectionId(config.connection_id).into());
    }
    if config.ibc_transfer_port_id != src_port_id {
        return Err(IbcError::InvalidPortId(config.ibc_transfer_port_id).into());
    }
    Ok(())
}
