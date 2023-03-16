#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Deps, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse,
    IbcEndpoint, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
    StdError, StdResult, SubMsg, WasmMsg,
};
#[cfg(not(test))]
use cosmwasm_std::{IbcQuery, QueryRequest};

#[cfg(not(test))]
use vectis_wallet::get_items_from_dao;

#[cfg(test)]
use crate::tests_ibc::get_items_from_dao;

use vectis_wallet::{
    check_ibc_order, check_ibc_version, DaoActors, GovecExecuteMsg, IbcError, PacketMsg,
    PrePropExecuteMsg, ProposalExecuteMsg, ProposeMessage, RemoteTunnelPacketMsg, StakeExecuteMsg,
    StdAck, VectisDaoActionIds, IBC_APP_VERSION,
};

use crate::state::IBC_TUNNELS;
use crate::ContractError;

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
    check_ibc_order(&channel.order)?;
    // In ibcv3 we don't check the version string passed in the IbcChannel
    // and only check the counterparty version in OpenTry
    if let Some(counter_version) = msg.counterparty_version() {
        check_ibc_version(counter_version)?;
    }
    // We return the version we need (which could be different than the counterparty version)
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
    // We currently do not save the channel_id to call the remote_tunnels
    // This is because dao-tunnel mainly relay actions from remote-tunnels
    let channel = msg.channel();
    if IBC_TUNNELS
        .may_load(
            deps.storage,
            (
                channel.connection_id.clone(),
                channel.counterparty_endpoint.port_id.clone(),
            ),
        )?
        .is_none()
    {
        return Err(StdError::generic_err(
            ContractError::InvalidTunnel.to_string(),
        ));
    }
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", &channel.endpoint.channel_id)
        .add_attribute("src_port_id", &channel.counterparty_endpoint.port_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Multiple channels are supported so this is is just observed
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
) -> Result<IbcReceiveResponse, ContractError> {
    (|| {
        let packet = msg.packet;

        is_authorised_src(deps.as_ref(), packet.src, packet.dest)?;

        let packet_msg: PacketMsg =
            from_binary(&packet.data).map_err(|_| IbcError::InvalidPacketMsg)?;

        let remote_ibc_msg: RemoteTunnelPacketMsg =
            from_binary(&packet_msg.msg).map_err(|_| IbcError::InvalidInnerMsg)?;

        match remote_ibc_msg {
            RemoteTunnelPacketMsg::MintGovec { wallet_addr } => {
                receive_mint_govec(deps, wallet_addr)
            }
            RemoteTunnelPacketMsg::GovecActions(msg) => {
                receive_govec_actions(deps, packet_msg.sender, msg)
            }
            RemoteTunnelPacketMsg::StakeActions(msg) => {
                receive_stake_actions(deps, packet_msg.sender, msg)
            }
            RemoteTunnelPacketMsg::PreProposalActions {
                pre_prop_module_addr,
                msg,
            } => receive_pre_proposal_actions(packet_msg.sender, pre_prop_module_addr, msg),
            RemoteTunnelPacketMsg::ProposalActions {
                prop_module_addr,
                msg,
            } => receive_proposal_actions(packet_msg.sender, prop_module_addr, msg),
        }
    })()
    .or_else(|e| {
        Ok(IbcReceiveResponse::new().set_ack(StdAck::fail(format!("IBC Packet Error: {e}"))))
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let res = IbcBasicResponse::new();
    let original_packet: PacketMsg = from_binary(&msg.original_packet.data)?;
    let ack_result: StdAck = from_binary(&msg.acknowledgement.data)?;
    let success = match ack_result {
        StdAck::Result(id) => {
            let reply_id: u64 = from_binary(&id)?;
            // id maps to VectisDaoActionIds
            format!("Success: {reply_id}")
        }
        StdAck::Error(e) => e,
    };
    Ok(res
        .add_attribute("job_id", original_packet.job_id.to_string())
        .add_attribute("result", success))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let original_packet: PacketMsg = from_binary(&msg.packet.data)?;
    Ok(IbcBasicResponse::new()
        .add_attribute("job_id", original_packet.job_id.to_string())
        .add_attribute("action", "Ibc Timeout"))
}

// Utils for dao-actions

fn receive_mint_govec(
    deps: DepsMut,
    wallet_addr: String,
) -> Result<IbcReceiveResponse, ContractError> {
    let msg = to_binary(&GovecExecuteMsg::Mint {
        new_wallet: wallet_addr,
    })?;
    let govec = get_items_from_dao(deps.as_ref(), DaoActors::Govec)?;

    let msg = SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: govec,
            msg,
            funds: vec![],
        },
        VectisDaoActionIds::GovecMint as u64,
    );

    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .add_attribute("action", "vectis_dao_tunnel_receive_mint_govec"))
}

pub fn receive_govec_actions(
    deps: DepsMut,
    sender: String,
    govec_msg: GovecExecuteMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let govec_addr = get_items_from_dao(deps.as_ref(), DaoActors::Govec)?;
    let sub_msg = match govec_msg {
        GovecExecuteMsg::Transfer {
            recipient, amount, ..
        } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: govec_addr,
                msg: to_binary(&GovecExecuteMsg::Transfer {
                    recipient,
                    amount,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::GovecTransfer as u64,
        ),
        GovecExecuteMsg::Send {
            contract,
            amount,
            msg,
            ..
        } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: govec_addr,
                msg: to_binary(&GovecExecuteMsg::Send {
                    contract,
                    amount,
                    msg,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::GovecSend as u64,
        ),
        GovecExecuteMsg::Exit { .. } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: govec_addr,
                msg: to_binary(&GovecExecuteMsg::Exit {
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::GovecExit as u64,
        ),
        _ => return Err(ContractError::Unauthorized {}),
    };

    // Ack is set in the reply
    Ok(IbcReceiveResponse::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "vectis_tunnel_receive_govec_actions"))
}

pub fn receive_stake_actions(
    deps: DepsMut,
    sender: String,
    msg: StakeExecuteMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let staking_addr = get_items_from_dao(deps.as_ref(), DaoActors::Staking)?;
    let sub_msg = match msg {
        StakeExecuteMsg::Unstake { amount, .. } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: staking_addr,
                msg: to_binary(&StakeExecuteMsg::Unstake {
                    amount,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::StakeUnstake as u64,
        ),
        StakeExecuteMsg::Claim { .. } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: staking_addr,
                msg: to_binary(&StakeExecuteMsg::Claim {
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::StakeClaim as u64,
        ),
        _ => return Err(ContractError::Unauthorized {}),
    };

    // Ack is set in the reply
    Ok(IbcReceiveResponse::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "vectis_tunnel_receive_stake_actions"))
}

pub fn receive_pre_proposal_actions(
    sender: String,
    pre_prop_module_addr: String,
    msg: PrePropExecuteMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let sub_msg = match msg {
        PrePropExecuteMsg::Propose {
            msg:
                ProposeMessage::Propose {
                    title,
                    description,
                    msgs,
                    ..
                },
        } => {
            let filled_pre_prop = PrePropExecuteMsg::Propose {
                msg: ProposeMessage::Propose {
                    title,
                    description,
                    msgs,
                    relayed_from: Some(sender),
                },
            };
            SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: pre_prop_module_addr.clone(),
                    msg: to_binary(&filled_pre_prop)?,
                    funds: vec![],
                },
                VectisDaoActionIds::PrePropExecute as u64,
            )
        }
        _ => return Err(ContractError::InvalidMsg("PreProposal".to_string())),
    };

    // Ack is set in the reply
    Ok(IbcReceiveResponse::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "vectis_tunnel_receive_pre_proposal_actions")
        .add_attribute("pre prop module addr", pre_prop_module_addr))
}

pub fn receive_proposal_actions(
    sender: String,
    prop_module_addr: String,
    msg: ProposalExecuteMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let sub_msg = match msg {
        ProposalExecuteMsg::Vote {
            proposal_id,
            vote,
            rationale,
            ..
        } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: prop_module_addr.clone(),
                msg: to_binary(&ProposalExecuteMsg::Vote {
                    proposal_id,
                    vote,
                    rationale,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::ProposalVote as u64,
        ),
        ProposalExecuteMsg::UpdateRationale {
            proposal_id,
            rationale,
            ..
        } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: prop_module_addr.clone(),
                msg: to_binary(&ProposalExecuteMsg::UpdateRationale {
                    proposal_id,
                    rationale,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::ProposalUpdateRationale as u64,
        ),
        ProposalExecuteMsg::Execute { proposal_id, .. } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: prop_module_addr.clone(),
                msg: to_binary(&ProposalExecuteMsg::Execute {
                    proposal_id,
                    relayed_from: Some(sender),
                })?,
                funds: vec![],
            },
            VectisDaoActionIds::ProposalExecute as u64,
        ),
        ProposalExecuteMsg::Close { proposal_id, .. } => SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: prop_module_addr.clone(),
                msg: to_binary(&ProposalExecuteMsg::Close { proposal_id })?,
                funds: vec![],
            },
            VectisDaoActionIds::ProposalClose as u64,
        ),
        _ => return Err(ContractError::Unauthorized {}),
    };

    // Ack is set in the reply
    Ok(IbcReceiveResponse::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "vectis_tunnel_receive_proposal_actions")
        .add_attribute("prop module addr", prop_module_addr))
}

/// Query for the correct connection_id of the underlying light client
#[cfg(not(test))]
fn is_authorised_src(
    deps: Deps,
    counterparty_endpoint: IbcEndpoint,
    endpoint: IbcEndpoint,
) -> Result<(), ContractError> {
    use cosmwasm_std::ChannelResponse;

    let channel_resp: ChannelResponse =
        deps.querier.query(&QueryRequest::Ibc(IbcQuery::Channel {
            channel_id: endpoint.channel_id,
            port_id: Some(endpoint.port_id),
        }))?;

    let local_connection_id = channel_resp
        .channel
        .ok_or(ContractError::InvalidTunnel)?
        .connection_id;

    if IBC_TUNNELS
        .may_load(
            deps.storage,
            (local_connection_id, counterparty_endpoint.port_id),
        )?
        .is_none()
    {
        return Err(ContractError::InvalidTunnel);
    }

    Ok(())
}

#[cfg(test)]
fn is_authorised_src(
    deps: Deps,
    counterparty_endpoint: IbcEndpoint,
    _endpoint: IbcEndpoint,
) -> Result<(), ContractError> {
    let local_connection_id = "TEST_CONNECTION_ID".to_string();

    if IBC_TUNNELS
        .may_load(
            deps.storage,
            (local_connection_id, counterparty_endpoint.port_id),
        )?
        .is_none()
    {
        return Err(ContractError::InvalidTunnel);
    }

    Ok(())
}
