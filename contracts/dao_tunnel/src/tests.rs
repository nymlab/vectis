use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_ack, mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, to_binary, Attribute, Coin, CosmosMsg, DepsMut, Empty, Ibc3ChannelOpenResponse,
    IbcAcknowledgement, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcOrder, OwnedDeps,
    Reply, StdError, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};

use vectis_wallet::{
    DaoTunnelPacketMsg, GovecExecuteMsg, IbcError, PacketMsg, ProposalExecuteMsg,
    RemoteTunnelPacketMsg, StdAck, VectisDaoActionIds,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER, IBC_APP_VERSION,
    PACKET_LIFETIME,
};
use voting::Vote;

use crate::contract::{execute, instantiate, query_controllers, query_govec, reply};
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_ack, ibc_packet_receive};
use crate::msg::{ExecuteMsg, InstantiateMsg, RemoteTunnels};
use crate::state::ADMIN;
use crate::ContractError;

const TEST_CONNECTION_ID: &str = "TEST_CONNECTION_ID";
const CHANNEL_ID: &str = "channel-1";
// To match src port id in `mock_ibc_packet_recv`
// For testing `ibc_packet_receive`_
const SRC_PORT_ID_RCV: &str = "their-port";
// To match src port id in `mock_ibc_channel`
// For testing `ibc_channel_connect`
const SRC_PORT_ID_CONNECT: &str = "their_port";

const ADMIN_ADDR: &str = "admin_addr";
const GOVEC_ADDR: &str = "govec_addr";
const UPDATE_CHANNEL_JOB_ID: u64 = 8721;
const INST_FACTORY_JOB_ID: u64 = 8711;
const WALLET_ADDR: &str = "proxy_wallet_addr";

fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN_ADDR, &[]);
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        govec_minter: GOVEC_ADDR.to_string(),
    };

    instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    let admin_addr = ADMIN.load(&deps.storage).unwrap();

    assert_eq!(
        deps.as_mut().api.addr_humanize(&admin_addr).unwrap(),
        ADMIN_ADDR.to_string()
    );

    let res = query_govec(deps.as_ref()).unwrap().unwrap();

    assert_eq!(res.to_string(), GOVEC_ADDR.to_string());

    deps
}

fn add_mock_controller(mut deps: DepsMut, src_port_id: &str) {
    let env = mock_env();
    execute(
        deps.branch(),
        env.clone(),
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

// Tests for contract ExecuteMsg functions
#[test]
fn only_admin_can_add_controllers() {
    let mut deps = do_instantiate();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("RANDOM_ADDR", &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: TEST_CONNECTION_ID.to_string(),
            port_id: SRC_PORT_ID_CONNECT.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: TEST_CONNECTION_ID.to_string(),
            port_id: SRC_PORT_ID_CONNECT.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(
                TEST_CONNECTION_ID.to_string(),
                SRC_PORT_ID_CONNECT.to_string()
            )]
        },
        res
    );
}

#[test]
fn only_admin_can_update_remote_tunnel_channel() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let invalid_sender = "RANDOM_ADDR";
    let valid_sender = ADMIN_ADDR;
    let channel_id = "new_channel_id";

    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(invalid_sender, &[]),
        ExecuteMsg::UpdateRemoteTunnelChannel {
            channel_id: channel_id.to_string(),
            job_id: UPDATE_CHANNEL_JOB_ID,
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(valid_sender, &[]),
        ExecuteMsg::UpdateRemoteTunnelChannel {
            channel_id: channel_id.to_string(),
            job_id: UPDATE_CHANNEL_JOB_ID,
        },
    )
    .unwrap();
    let msg: CosmosMsg<Empty> = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: channel_id.to_string(),
        data: to_binary(&PacketMsg {
            // This should be the contract address
            sender: env.contract.address.to_string(),
            job_id: UPDATE_CHANNEL_JOB_ID,
            msg: to_binary(&DaoTunnelPacketMsg::UpdateChannel {}).unwrap(),
        })
        .unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
    assert_eq!(
        res.attributes,
        vec![("action", "update_remote_tunnel_channel"),]
    )
}

#[test]
fn only_admin_can_instantiate_factory() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let invalid_sender = "RANDOM_ADDR";
    let valid_sender = ADMIN_ADDR;

    let instantiation_msg = FactoryInstantiateMsg {
        addr_prefix: "prefix".to_string(),
        govec_minter: None,
        proxy_code_id: 45,
        proxy_multisig_code_id: 23,
        wallet_fee: Coin {
            denom: "denom".to_string(),
            amount: Uint128::MAX,
        },
    };

    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(invalid_sender, &[]),
        ExecuteMsg::InstantiateRemoteFactory {
            job_id: INST_FACTORY_JOB_ID,
            code_id: 45,
            msg: instantiation_msg.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(valid_sender, &[]),
        ExecuteMsg::InstantiateRemoteFactory {
            job_id: INST_FACTORY_JOB_ID,
            code_id: 45,
            msg: instantiation_msg.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap();

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&PacketMsg {
            job_id: INST_FACTORY_JOB_ID,
            sender: env.contract.address.to_string(),
            msg: to_binary(&DaoTunnelPacketMsg::InstantiateFactory {
                code_id: 45,
                msg: instantiation_msg,
            })
            .unwrap(),
        })
        .unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
    assert_eq!(
        res.attributes,
        vec![("action", "execute_instantiate_remote_factory")]
    )
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
        mock_ibc_channel_open_init(CHANNEL_ID, APP_ORDER, "some-invalid-version");
    let res = ibc_channel_open(deps.as_mut(), mock_env(), invalid_ibc_channel_open_msg)
        .unwrap()
        .unwrap();
    assert_eq!(res.version, IBC_APP_VERSION);

    // In open try, counterparty_version is checked
    // invalid version
    let invalid_ibc_channel_open_try_msg =
        mock_ibc_channel_open_try(CHANNEL_ID, APP_ORDER, "some-invalid-version");
    let err =
        ibc_channel_open(deps.as_mut(), mock_env(), invalid_ibc_channel_open_try_msg).unwrap_err();

    assert_eq!(err, IbcError::InvalidChannelVersion(IBC_APP_VERSION).into());

    let valid_ibc_channel_open_try_msg =
        mock_ibc_channel_open_try(CHANNEL_ID, APP_ORDER, IBC_APP_VERSION);
    let res = ibc_channel_open(deps.as_mut(), mock_env(), valid_ibc_channel_open_try_msg).unwrap();

    assert_eq!(
        res,
        Some(Ibc3ChannelOpenResponse {
            version: IBC_APP_VERSION.to_string()
        })
    );
}

// Tests for `ibc_channel_connect`
// Same message for ack and confirm (counterparty_version was check in open)
#[test]
fn only_approved_endpoint_can_open_and_connect() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut(), SRC_PORT_ID_CONNECT);

    if let IbcChannelConnectMsg::OpenAck {
        mut channel,
        counterparty_version,
    } = mock_ibc_channel_connect_ack(CHANNEL_ID, APP_ORDER, IBC_APP_VERSION)
    {
        // port id not added to DAO_TUNNEL
        channel.counterparty_endpoint.port_id = SRC_PORT_ID_RCV.to_string();
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

        // port id is added to DAO_TUNNEL
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

    let incorrect_ibc_msg = DaoTunnelPacketMsg::UpdateChannel {};

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
        // remote tunnel expects DaoTunnelPacketMsg
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

// Tests for dispatching dao actions
// (after checks for `ibc_packet_receive` has been passed)

#[test]
fn recieve_mint_govec_works() {
    let mut deps = do_instantiate();
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id: 879,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: WALLET_ADDR.to_string(),
        })
        .unwrap(),
    };

    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);
    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let msg = to_binary(&GovecExecuteMsg::Mint {
        new_wallet: WALLET_ADDR.to_string(),
    })
    .unwrap();

    let sub_msg = WasmMsg::Execute {
        contract_addr: GOVEC_ADDR.to_string(),
        msg,
        funds: vec![],
    };

    // test the right submessage is created
    assert!(!res.attributes.is_empty());
    assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(sub_msg));

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

#[test]
fn recieve_burn_govec_works() {
    let mut deps = do_instantiate();
    let valid_govec_action = GovecExecuteMsg::Burn {
        relayed_from: Some("addr_not_to_be_used".to_string()),
    };
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id: 879,
        msg: to_binary(&RemoteTunnelPacketMsg::GovecActions(valid_govec_action)).unwrap(),
    };

    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let sub_msg = WasmMsg::Execute {
        contract_addr: GOVEC_ADDR.to_string(),
        msg: to_binary(&GovecExecuteMsg::Burn {
            relayed_from: Some(WALLET_ADDR.to_string()),
        })
        .unwrap(),
        funds: vec![],
    };
    assert!(!res.attributes.is_empty());
    assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(sub_msg));

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
    assert_eq!(ack, VectisDaoActionIds::GovecBurn as u64)
}

#[test]
fn recieve_proposal_vote_works() {
    let mut deps = do_instantiate();
    let prop_module_addr = "Some Prop Addr".to_string();
    let valid_proposal_action = ProposalExecuteMsg::Vote {
        proposal_id: 1,
        vote: Vote::Yes,
        relayed_from: Some("addr_not_to_be_used".to_string()),
    };
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id: 879,
        msg: to_binary(&RemoteTunnelPacketMsg::ProposalActions {
            prop_module_addr: prop_module_addr.clone(),
            msg: valid_proposal_action,
        })
        .unwrap(),
    };

    add_mock_controller(deps.as_mut(), SRC_PORT_ID_RCV);

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let sub_msg = WasmMsg::Execute {
        contract_addr: prop_module_addr,
        msg: to_binary(&ProposalExecuteMsg::Vote {
            proposal_id: 1,
            vote: Vote::Yes,
            relayed_from: Some(WALLET_ADDR.to_string()),
        })
        .unwrap(),
        funds: vec![],
    };
    assert!(!res.attributes.is_empty());
    assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(sub_msg));

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
    assert_eq!(ack, VectisDaoActionIds::ProposalVote as u64)
}

// Tests for mock_ibc_packet_ack

#[test]
fn ack_emits_job_id() {
    let job_id: u64 = 45;
    let reply_id: u64 = 44;
    let mut deps = do_instantiate();
    let env = mock_env();
    let ack = StdAck::success(reply_id);
    let ibc_msg = PacketMsg {
        sender: WALLET_ADDR.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::GovecActions(
            GovecExecuteMsg::Burn {
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
