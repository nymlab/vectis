#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, CosmosMsg, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdError, StdResult, SubMsg,
    WasmMsg,
};

use vectis_wallet::{
    check_order, check_version, ChainConfig, DaoConfig, DaoTunnelPacketMsg, IbcError, PacketMsg,
    RemoteTunnelPacketMsg, StdAck, VectisDaoActionIds, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, IBC_APP_VERSION,
};

use crate::state::{CHAIN_CONFIG, DAO_CONFIG, IBC_TRANSFER_MODULES};
use crate::{ContractError, DISPATCH_CALLBACK_ID, FACTORY_CALLBACK_ID};

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
    let mut dao_config = DAO_CONFIG.load(deps.storage)?;

    let remote_port_id = channel.counterparty_endpoint.port_id.clone();
    let local_channel_id = channel.endpoint.channel_id.clone();

    if ensure_is_dao_tunnel(
        &dao_config,
        &channel.connection_id,
        &channel.counterparty_endpoint.port_id,
    )
    .is_ok()
    {
        if dao_config.dao_tunnel_channel.is_none() {
            // This addresses the case for first channel and subsequently when a channel has been
            // closed and a new one needs to be created
            dao_config.dao_tunnel_channel = Some(local_channel_id.clone());
            DAO_CONFIG.save(deps.storage, &dao_config)?;
            Ok(IbcBasicResponse::new()
                .add_attribute("action", "ibc_connect")
                .add_attribute("SAVED local dao_tunnel channel_id", &local_channel_id)
                .add_attribute("dao_tunnel_port_id", &remote_port_id))
        } else {
            // We accept new channel creation but this will only be used if the DAO calls
            // `UpdateDaoConfig` to update the official channel used to communicate with the
            // dao-tunnel
            Ok(IbcBasicResponse::new()
                .add_attribute("action", "ibc_connect")
                .add_attribute("IGNORED local dao_tunnel channel_id", &local_channel_id)
                .add_attribute("dao_tunnel_port_id", &remote_port_id))
        }
    } else {
        Err(StdError::GenericErr {
            msg: ContractError::Unauthorized.to_string(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Channel close
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let mut dao_config = DAO_CONFIG.load(deps.storage)?;
    let channel = msg.channel();
    let connection_id = channel.connection_id.clone();

    if let Some(current_channel) = dao_config.dao_tunnel_channel {
        if channel.endpoint.channel_id == current_channel {
            dao_config.dao_tunnel_channel = None;
            DAO_CONFIG.save(deps.storage, &dao_config)?;
        }
    } else {
        IBC_TRANSFER_MODULES.remove(deps.storage, connection_id);
    }

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
        let dao_channel = DAO_CONFIG
            .load(deps.storage)?
            .dao_tunnel_channel
            .ok_or(ContractError::DaoChannelNotFound)?;

        // We only need to check for dao_channel here because messages can only be received from
        // authorised / opened channels, which for a remote_tunnel,
        // only connects to the dao_tunnel_channel is expected to recieve messages
        if msg.packet.dest.channel_id == dao_channel {
            match dao_tunnel_msg {
                DaoTunnelPacketMsg::UpdateDaoConfig { new_config } => {
                    receive_update_dao_config(deps, new_config, packet_msg.job_id)
                }
                DaoTunnelPacketMsg::UpdateChainConfig { new_config } => {
                    receive_update_chain_config(deps, new_config, packet_msg.job_id)
                }
                DaoTunnelPacketMsg::InstantiateFactory { code_id, msg } => {
                    receive_instantiate(deps, code_id, msg, packet_msg.job_id)
                }
                DaoTunnelPacketMsg::UpdateIbcTransferRecieverChannel {
                    connection_id,
                    channel,
                } => receive_update_ibc_transfer_modules(
                    deps,
                    connection_id,
                    channel,
                    packet_msg.job_id,
                ),
                DaoTunnelPacketMsg::DispatchActions { msgs } => {
                    receive_dispatch_actions(msgs, packet_msg.job_id)
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
            .add_attribute("job_id", original_packet_data.job_id.to_string())
            .add_attribute("action", "Mint Govec Ack")
            .add_attribute("success", success.to_string())
            .add_submessage(submsg))
    } else {
        let success = match ack_result {
            StdAck::Result(id) => {
                // id maps to VectisDaoActionIds
                let action: u64 = from_binary(&id)?;
                format!("{:?}", VectisDaoActionIds::try_from(action)?)
            }
            StdAck::Error(e) => e,
        };
        Ok(res
            .add_attribute("job_id", original_packet_data.job_id.to_string())
            .add_attribute("result", success))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// revert the state of minting govec
/// return transfers
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let res = IbcBasicResponse::new();
    let original_packet: PacketMsg = from_binary(&msg.packet.data)?;

    if let RemoteTunnelPacketMsg::MintGovec { wallet_addr } = from_binary(&original_packet.msg)? {
        let submsg = SubMsg::new(WasmMsg::Execute {
            contract_addr: original_packet.sender,
            msg: to_binary(&WalletFactoryExecuteMsg::GovecMinted {
                success: false,
                wallet_addr,
            })?,
            funds: vec![],
        });

        Ok(res
            .add_attribute("job_id", original_packet.job_id.to_string())
            .add_attribute("action", "Mint Govec Timeout: revert")
            .add_submessage(submsg))
    } else {
        Ok(res
            .add_attribute("job_id", original_packet.job_id.to_string())
            .add_attribute("action", "Timeout"))
    }
}

// Recieve handlers

pub fn receive_update_dao_config(
    deps: DepsMut,
    new_config: DaoConfig,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    DAO_CONFIG.save(deps.storage, &new_config)?;
    Ok(IbcReceiveResponse::new()
        .set_ack(StdAck::success(job_id))
        .add_attribute("action", "dao config updated"))
}

pub fn receive_update_chain_config(
    deps: DepsMut,
    new_config: ChainConfig,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    CHAIN_CONFIG.save(deps.storage, &new_config)?;
    Ok(IbcReceiveResponse::new()
        .set_ack(StdAck::success(job_id))
        .add_attribute("action", "chain config updated"))
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

pub fn receive_update_ibc_transfer_modules(
    deps: DepsMut,
    connection_id: String,
    channel: Option<String>,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    match channel {
        Some(c) => {
            // Update the channel
            IBC_TRANSFER_MODULES.save(deps.storage, connection_id, &c)?;
        }
        None => {
            // Remove it
            IBC_TRANSFER_MODULES.remove(deps.storage, connection_id);
        }
    }
    let res = IbcReceiveResponse::new()
        .set_ack(StdAck::success(job_id))
        .add_attribute("action", "ibc transfer module updated");
    Ok(res)
}

pub fn receive_dispatch_actions(
    msgs: Vec<CosmosMsg>,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    let acknowledgement = StdAck::success(job_id);

    let sub_msgs: Vec<SubMsg> = msgs
        .iter()
        .map(|m| SubMsg::reply_on_error(m.clone(), DISPATCH_CALLBACK_ID))
        .collect();

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessages(sub_msgs)
        .add_attribute("action", "vectis_tunnel_remote_receive_dispatch"))
}

// utils
fn ensure_is_dao_tunnel(
    dao_config: &DaoConfig,
    local_connection_id: &str,
    src_port_id: &str,
) -> Result<(), ContractError> {
    if dao_config.connection_id != local_connection_id {
        return Err(IbcError::InvalidConnectionId(local_connection_id.into()).into());
    }
    if dao_config.dao_tunnel_port_id != src_port_id {
        return Err(IbcError::InvalidPortId(src_port_id.into()).into());
    }
    Ok(())
}
