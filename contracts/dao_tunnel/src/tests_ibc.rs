use cosmwasm_std::{
    testing::mock_ibc_channel_close_init, wasm_execute, IbcChannelCloseMsg, IbcReceiveResponse,
    Response,
};

use crate::{ibc::ibc_channel_close, tests::*};

// Utils
pub fn add_mock_controller(mut deps: DepsMut, src_port_id: &str) {
    let env = mock_env();
    execute(
        deps.branch(),
        env,
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: TEST_CONNECTION_ID.to_string(),
            port_id: src_port_id.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(TEST_CONNECTION_ID.to_string(), src_port_id.to_string())]
        },
        res
    );
}

// Tests for `ibc_channel_open`

// Utils for `ibc_channel_open`
fn mock_ibc_channel_open_init(
    channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    let channel = mock_ibc_channel(channel_id, order, version);
    IbcChannelOpenMsg::new_init(channel)
}

fn mock_ibc_channel_open_try(
    channel_id: &str,
    order: IbcOrder,
    counterparty_version: &str,
) -> IbcChannelOpenMsg {
    let channel = mock_ibc_channel(channel_id, order, counterparty_version);
    IbcChannelOpenMsg::new_try(channel, counterparty_version)
}

#[test]
fn only_correct_version_order_can_open() {
    let mut deps = do_instantiate();

    // Open init
    // invalid order
    let invalid_ibc_channel_open_msg =
        mock_ibc_channel_open_init(CHANNEL_ID, IbcOrder::Ordered, IBC_APP_VERSION);
    let err =
        ibc_channel_open(deps.as_mut(), mock_env(), invalid_ibc_channel_open_msg).unwrap_err();
    assert_eq!(err, IbcError::InvalidChannelOrder.into());

    // In open init, invalid version is not not checked in IBCv3
    let invalid_ibc_channel_open_msg =
        mock_ibc_channel_open_init(CHANNEL_ID, IBC_APP_ORDER, "some-invalid-version");
    let res = ibc_channel_open(deps.as_mut(), mock_env(), invalid_ibc_channel_open_msg)
        .unwrap()
        .unwrap();
    assert_eq!(res.version, IBC_APP_VERSION);

    // In open try, counterparty_version is checked
    // invalid version
    let invalid_ibc_channel_open_try_msg =
        mock_ibc_channel_open_try(CHANNEL_ID, IBC_APP_ORDER, "some-invalid-version");
    let err =
        ibc_channel_open(deps.as_mut(), mock_env(), invalid_ibc_channel_open_try_msg).unwrap_err();

    assert_eq!(err, IbcError::InvalidChannelVersion(IBC_APP_VERSION).into());

    let valid_ibc_channel_open_try_msg =
        mock_ibc_channel_open_try(CHANNEL_ID, IBC_APP_ORDER, IBC_APP_VERSION);
    let res = ibc_channel_open(deps.as_mut(), mock_env(), valid_ibc_channel_open_try_msg).unwrap();

    assert_eq!(
        res,
        Some(Ibc3ChannelOpenResponse {
            version: IBC_APP_VERSION.to_string()
        })
    );
}

// Tests for `ibc_channel_close`
#[test]
fn correct_event_emitted_when_channel_closes() {
    let mut deps = do_instantiate();
    let init_msg = mock_ibc_channel_close_init(CHANNEL_ID, IBC_APP_ORDER, IBC_APP_VERSION);
    let mut channel = init_msg.channel().to_owned();
    channel.connection_id = TEST_CONNECTION_ID.to_string();
    channel.endpoint.channel_id = CHANNEL_ID.to_string();
    channel.counterparty_endpoint.port_id = SRC_PORT_ID_RCV.to_string();
    let msg = IbcChannelCloseMsg::new_init(channel);
    let res = ibc_channel_close(deps.as_mut(), mock_env(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("action", "ibc_close"),
            ("channel_id", CHANNEL_ID),
            ("src_port_id", SRC_PORT_ID_RCV),
            ("connection_id", TEST_CONNECTION_ID)
        ]
    )
}

// Tests for `ibc_channel_connect`
// Same message for ack and confirm (counterparty_version was check in open)
#[test]
fn only_approved_endpoint_can_connect() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_CONNECT);

    if let IbcChannelConnectMsg::OpenAck {
        mut channel,
        counterparty_version,
    } = mock_ibc_channel_connect_ack(CHANNEL_ID, IBC_APP_ORDER, IBC_APP_VERSION)
    {
        // port id not added to DAO_TUNNEL
        channel.counterparty_endpoint.port_id = "SOME PORT_ID".to_string();
        channel.connection_id = TEST_CONNECTION_ID.to_string();
        let err = ibc_channel_connect(
            deps.as_mut(),
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel: channel.clone(),
                counterparty_version: counterparty_version.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            StdError::GenericErr {
                msg: "Invalid remote tunnel".to_string()
            }
        );

        // connection_id not added to DAO_TUNNEL
        channel.counterparty_endpoint.port_id = SRC_PORT_ID_CONNECT.to_string();
        channel.connection_id = "SOME CONNECTION".to_string();
        let err = ibc_channel_connect(
            deps.as_mut(),
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel: channel.clone(),
                counterparty_version: counterparty_version.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            StdError::GenericErr {
                msg: "Invalid remote tunnel".to_string()
            }
        );

        // port id + connection_id is added to DAO_TUNNEL
        channel.counterparty_endpoint.port_id = SRC_PORT_ID_CONNECT.to_string();
        channel.connection_id = TEST_CONNECTION_ID.to_string();
        let res = ibc_channel_connect(
            deps.as_mut(),
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel,
                counterparty_version,
            },
        )
        .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "ibc_connect"),
                Attribute::new("channel_id", CHANNEL_ID),
                Attribute::new("src_port_id", SRC_PORT_ID_CONNECT)
            ]
        );
    } else {
        assert!(false)
    };
}

// Tests for `ibc_packet_receive`
#[test]
fn returns_ack_failure_when_invalid_ibc_packet_msg() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let incorrect_ibc_msg = &[1; 11];

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &incorrect_ibc_msg).unwrap();
    // This function cannot error
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    if let StdAck::Error(m) = from_binary(&res.acknowledgement).unwrap() {
        assert_eq!(
            m,
            format!("IBC Packet Error: {}", IbcError::InvalidPacketMsg)
        );
    } else {
        assert!(false)
    }
}

#[test]
fn returns_ack_failure_when_invalid_inner_remote_tunnel_msg() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);
    let incorrect_inner_msg = PacketMsg {
        sender: ADMIN_ADDR.to_string(),
        job_id: 1,
        // dao tunnel expects RemoteTunnelPacketMsg
        msg: to_binary(&[2; 0]).unwrap(),
    };

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &incorrect_inner_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    if let StdAck::Error(m) = from_binary(&res.acknowledgement).unwrap() {
        assert_eq!(
            m,
            format!("IBC Packet Error: {}", IbcError::InvalidInnerMsg)
        );
    } else {
        assert!(false)
    }
}

#[test]
fn returns_ack_failure_unauthorised_source() {
    let mut deps = do_instantiate();
    let ibc_msg = &[1; 11];

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    if let StdAck::Error(m) = from_binary(&res.acknowledgement).unwrap() {
        assert_eq!(
            m,
            format!("IBC Packet Error: {}", ContractError::InvalidTunnel)
        );
    } else {
        assert!(false)
    }
}

// Tests for dispatching dao actions
// (after checks for `ibc_packet_receive` has been passed)

// RemoteTunnelPacketMsg::MintGovec
#[test]
fn recieve_mint_govec_works() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id: 879,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: WALLET_ADDR.to_string(),
        })
        .unwrap(),
    };
    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let expected_sub_msg = WasmMsg::Execute {
        contract_addr: GOVEC_ADDR.to_string(),
        msg: to_binary(&GovecExecuteMsg::Mint {
            new_wallet: WALLET_ADDR.to_string(),
        })
        .unwrap(),
        funds: vec![],
    };

    // test correct ack, attribute and submessage is created
    assert!(res.acknowledgement.is_empty());
    assert_eq!(
        res.attributes,
        vec![("action", "vectis_dao_tunnel_receive_mint_govec")]
    );
    assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(expected_sub_msg));
    assert_eq!(res.messages.len(), 1);

    // test set_data correctly overwrite ack to expected value -
    // The type of operation in VectisDaoActionIds
    let reply_msg = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: None,
        }),
    };

    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
    let ack: StdAck = from_binary(&res.data.unwrap()).unwrap();
    let ack: u64 = from_binary(&ack.unwrap()).unwrap();
    assert_eq!(ack, VectisDaoActionIds::GovecMint as u64)
}

// RemoteTunnelPacketMsg::GovecActions
//
fn test_rec_ibc_package(
    deps: DepsMut,
    msg: &RemoteTunnelPacketMsg,
    job_id: u64,
) -> Result<IbcReceiveResponse, ContractError> {
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id,
        msg: to_binary(msg).unwrap(),
    };

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    ibc_packet_receive(deps, mock_env(), msg)
}

fn test_reply(
    deps: DepsMut,
    reply_id: u64,
    reply_result: SubMsgResult,
) -> Result<Response, ContractError> {
    let reply_msg = Reply {
        id: reply_id,
        result: reply_result,
    };

    reply(deps, mock_env(), reply_msg)
}

// fn test_rcv_ibc_with_reply(deps: DepsMut, ibc_msg: &RemoteTunnelPacketMsg, job_id: u64, sender: &str, sub_msg)

#[test]
fn recieve_user_mint_govec_fails() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let res = test_rec_ibc_package(
        deps.as_mut(),
        &RemoteTunnelPacketMsg::GovecActions(GovecExecuteMsg::Mint {
            new_wallet: "addr_not_to_be_used".to_string(),
        }),
        JOB_ID,
    )
    .unwrap();

    match from_binary(&res.acknowledgement).unwrap() {
        StdAck::Error(m) => {
            assert_eq!(
                m,
                format!("IBC Packet Error: {}", ContractError::Unauthorized)
            );
        }
        StdAck::Result(_) => {
            assert!(false)
        }
    }
}

const SUBMSG_RES: SubMsgResult = SubMsgResult::Ok(SubMsgResponse {
    events: vec![],
    data: None,
});

#[test]
fn recieve_govec_actions_works() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    // Exit
    //
    // test match sub messages, attributes and ack
    let exit = GovecExecuteMsg::Exit {
        relayed_from: Some(WALLET_ADDR.to_string()),
    };
    let ibc_exit = RemoteTunnelPacketMsg::GovecActions(exit.clone());
    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_exit, JOB_ID).unwrap();

    let reply_res = test_reply(
        deps.as_mut(),
        VectisDaoActionIds::GovecExit as u64,
        SUBMSG_RES,
    )
    .unwrap();
    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(GOVEC_ADDR, &exit, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![("action", "vectis_tunnel_receive_govec_actions")]
    );
    assert_eq!(
        from_binary::<u64>(&ack).unwrap(),
        VectisDaoActionIds::GovecExit as u64
    );

    // transfer
    //
    // test match sub messages, attributes and ack
    let transfer = GovecExecuteMsg::Transfer {
        recipient: "someone".to_string(),
        amount: Uint128::one(),
        relayed_from: Some(WALLET_ADDR.to_string()),
    };
    let ibc_tranfser = RemoteTunnelPacketMsg::GovecActions(transfer.clone());
    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_tranfser, JOB_ID).unwrap();

    let reply_res = test_reply(
        deps.as_mut(),
        VectisDaoActionIds::GovecTransfer as u64,
        SUBMSG_RES,
    )
    .unwrap();
    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(GOVEC_ADDR, &transfer, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![("action", "vectis_tunnel_receive_govec_actions")]
    );
    assert_eq!(
        from_binary::<u64>(&ack).unwrap(),
        VectisDaoActionIds::GovecTransfer as u64
    );

    // Send
    //
    // test match sub messages, attributes and ack
    let send = GovecExecuteMsg::Send {
        contract: "some_contract".to_string(),
        amount: Uint128::one(),
        relayed_from: Some(WALLET_ADDR.to_string()),
        msg: Binary::from(&[11; 9]),
    };
    let ibc_send = RemoteTunnelPacketMsg::GovecActions(send.clone());
    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_send, JOB_ID).unwrap();

    let reply_res = test_reply(
        deps.as_mut(),
        VectisDaoActionIds::GovecSend as u64,
        SUBMSG_RES,
    )
    .unwrap();
    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(GOVEC_ADDR, &send, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![("action", "vectis_tunnel_receive_govec_actions")]
    );
    assert_eq!(
        from_binary::<u64>(&ack).unwrap(),
        VectisDaoActionIds::GovecSend as u64
    )
}

#[test]
fn invalid_proposal_action_fails() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let prop_module_addr = "Some Prop Addr";

    // This variant not forwarded
    let prop_msg = ProposalExecuteMsg::AddVoteHook {
        address: "some hooks".to_string(),
    };
    let ibc_propose = RemoteTunnelPacketMsg::ProposalActions {
        prop_module_addr: prop_module_addr.to_string(),
        msg: prop_msg,
    };

    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_propose, JOB_ID).unwrap();

    match from_binary(&rec_res.acknowledgement).unwrap() {
        StdAck::Error(m) => {
            assert_eq!(
                m,
                format!("IBC Packet Error: {}", ContractError::Unauthorized)
            );
        }
        StdAck::Result(_) => {
            assert!(false)
        }
    }
}

#[test]
fn recieve_proposal_actions_work() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let prop_module_addr = "Some Prop Addr";

    // Propose
    //
    // test match sub messages, attributes and ack
    let action_id = VectisDaoActionIds::ProposalPropose as u64;
    let propose = ProposalExecuteMsg::Propose {
        title: "title".to_string(),
        description: "des".to_string(),
        msgs: vec![],
        relayed_from: Some(WALLET_ADDR.to_string()),
    };

    let ibc_propose = RemoteTunnelPacketMsg::ProposalActions {
        prop_module_addr: prop_module_addr.to_string(),
        msg: propose.clone(),
    };

    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_propose, JOB_ID).unwrap();
    let reply_res = test_reply(deps.as_mut(), action_id, SUBMSG_RES).unwrap();

    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(prop_module_addr, &propose, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![
            ("action", "vectis_tunnel_receive_proposal_actions"),
            ("prop module addr", prop_module_addr),
        ]
    );
    assert_eq!(from_binary::<u64>(&ack).unwrap(), action_id);

    // Close
    let action_id = VectisDaoActionIds::ProposalClose as u64;
    let close = ProposalExecuteMsg::Close {
        proposal_id: 12,
        relayed_from: Some(WALLET_ADDR.to_string()),
    };

    let ibc_msg = RemoteTunnelPacketMsg::ProposalActions {
        prop_module_addr: prop_module_addr.to_string(),
        msg: close.clone(),
    };

    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_msg, JOB_ID).unwrap();
    let reply_res = test_reply(deps.as_mut(), action_id, SUBMSG_RES).unwrap();

    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(prop_module_addr, &close, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![
            ("action", "vectis_tunnel_receive_proposal_actions"),
            ("prop module addr", prop_module_addr),
        ]
    );
    assert_eq!(from_binary::<u64>(&ack).unwrap(), action_id);

    // Vote
    let action_id = VectisDaoActionIds::ProposalVote as u64;
    let vote = ProposalExecuteMsg::Vote {
        proposal_id: 12,
        vote: Vote::Yes,
        relayed_from: Some(WALLET_ADDR.to_string()),
    };

    let ibc_vote = RemoteTunnelPacketMsg::ProposalActions {
        prop_module_addr: prop_module_addr.to_string(),
        msg: vote.clone(),
    };

    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_vote, JOB_ID).unwrap();
    let reply_res = test_reply(deps.as_mut(), action_id, SUBMSG_RES).unwrap();

    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(prop_module_addr, &vote, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![
            ("action", "vectis_tunnel_receive_proposal_actions"),
            ("prop module addr", prop_module_addr),
        ]
    );
    assert_eq!(from_binary::<u64>(&ack).unwrap(), action_id);

    // Execute
    let action_id = VectisDaoActionIds::ProposalExecute as u64;
    let exe = ProposalExecuteMsg::Execute {
        proposal_id: 12,
        relayed_from: Some(WALLET_ADDR.to_string()),
    };

    let ibc_exe = RemoteTunnelPacketMsg::ProposalActions {
        prop_module_addr: prop_module_addr.to_string(),
        msg: exe.clone(),
    };

    let rec_res = test_rec_ibc_package(deps.as_mut(), &ibc_exe, JOB_ID).unwrap();
    let reply_res = test_reply(deps.as_mut(), action_id, SUBMSG_RES).unwrap();

    let ack = from_binary::<StdAck>(&reply_res.data.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        rec_res.messages[0].msg,
        CosmosMsg::Wasm(wasm_execute(prop_module_addr, &exe, vec![]).unwrap())
    );
    assert_eq!(
        rec_res.attributes,
        vec![
            ("action", "vectis_tunnel_receive_proposal_actions"),
            ("prop module addr", prop_module_addr),
        ]
    );
    assert_eq!(from_binary::<u64>(&ack).unwrap(), action_id);
}

// Tests for mock_ibc_packet_ack

#[test]
fn ack_emits_reply_id() {
    let job_id: u64 = 45;
    let reply_id: u64 = 44;
    let mut deps = do_instantiate();
    let env = mock_env();
    let ack = StdAck::success(reply_id);
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::GovecActions(
            GovecExecuteMsg::Exit {
                relayed_from: Some(WALLET_ADDR.to_string()),
            },
        ))
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_ack(CHANNEL_ID, &ibc_msg, IbcAcknowledgement::new(ack)).unwrap();
    let res = ibc_packet_ack(deps.as_mut(), env, ibc_ack).unwrap();

    assert_eq!(res.attributes[0].value, job_id.to_string());
    assert_eq!(res.attributes[1].value, format!("Success: {}", reply_id));
}

#[test]
fn handle_timeout() {
    let mut deps = do_instantiate();
    let job_id = 2901;
    let env = mock_env();

    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&[1; 1]).unwrap(),
    };

    let ibc_ack = mock_ibc_packet_timeout(CHANNEL_ID, &original_ibc_msg).unwrap();
    let res = ibc_packet_timeout(deps.as_mut(), env, ibc_ack).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("action", "Ibc Timeout".to_string())
        ]
    )
}
