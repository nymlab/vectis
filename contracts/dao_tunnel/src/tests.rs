use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, to_binary, Attribute, Coin, CosmosMsg, DepsMut, Empty, IbcChannelOpenMsg, IbcMsg,
    IbcOrder, OwnedDeps, Reply, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};

use vectis_wallet::{
    DaoTunnelPacketMsg, GovecExecuteMsg, IbcError, PacketMsg, RemoteTunnelPacketMsg, StdAck,
    VectisDaoActionIds, WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER,
    IBC_APP_VERSION, PACKET_LIFETIME,
};

use crate::contract::{execute, instantiate, query_controllers, query_govec, reply};
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
use crate::msg::{ExecuteMsg, InstantiateMsg, RemoteTunnels};
use crate::state::ADMIN;
use crate::ContractError;

const TEST_CONNECTION_ID: &str = "TEST_CONNECTION_ID";
const CHANNEL_ID: &str = "channel-1";
// from `mock_ibc_packet_recv`
const SRC_PORT_ID: &str = "their-port";
const ADMIN_ADDR: &str = "admin_addr";
const GOVEC_ADDR: &str = "govec_addr";
const UPDATE_CHANNEL_JOB_ID: u64 = 8721;
const INST_FACTORY_JOB_ID: u64 = 8711;

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

// TODO make these tests
// fn mock_ibc_channel_open_init(
//     connection_id: &str,
//     port_id: &str,
//     channel_id: &str,
//     order: IbcOrder,
//     version: &str,
// ) -> IbcChannelOpenMsg {
//     let mut channel = mock_ibc_channel(channel_id, order, version);
//     channel.connection_id = connection_id.to_string();
//     channel.endpoint.port_id = port_id.to_string();
//     IbcChannelOpenMsg::new_init(channel)
// }
// #[test]
// fn connect(mut deps: DepsMut, channel_id: &str) {
//     let handshake_open = mock_ibc_channel_open_init(
//         TEST_CONNECTION_ID,
//         SRC_PORT_ID,
//         channel_id,
//         APP_ORDER,
//         IBC_APP_VERSION,
//     );
//
//     ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();
//
//     let handshake_connect = mock_ibc_channel_connect_ack(channel_id, APP_ORDER, IBC_APP_VERSION);
//
//     let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();
//
//     assert_eq!(
//         res.attributes,
//         vec![
//             Attribute::new("action", "ibc_connect"),
//             Attribute::new("channel_id", channel_id),
//             Attribute::new("src_port_id", "their_port")
//         ]
//     );
// }

fn add_mock_controller(mut deps: DepsMut) {
    let env = mock_env();
    execute(
        deps.branch(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: TEST_CONNECTION_ID.to_string(),
            port_id: SRC_PORT_ID.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(TEST_CONNECTION_ID.to_string(), SRC_PORT_ID.to_string())]
        },
        res
    );
}

#[test]
fn only_admin_can_add_controllers() {
    let mut deps = do_instantiate();

    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("RANDOM_ADDR", &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: TEST_CONNECTION_ID.to_string(),
            port_id: SRC_PORT_ID.to_string(),
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
            port_id: SRC_PORT_ID.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(TEST_CONNECTION_ID.to_string(), SRC_PORT_ID.to_string())]
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

// Tests for `ibc_packet_receive`

#[test]
fn returns_ack_failure_when_invalid_ibc_packet_msg() {
    let mut deps = do_instantiate();
    add_mock_controller(deps.as_mut());

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
    add_mock_controller(deps.as_mut());
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

#[test]
fn handle_recieve_mint_govec() {
    let mut deps = do_instantiate();
    let wallet_addr = "proxy_wallet_addr";
    let ibc_msg = PacketMsg {
        sender: wallet_addr.to_string(),
        job_id: 879,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: wallet_addr.to_string(),
        })
        .unwrap(),
    };

    add_mock_controller(deps.as_mut());

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    assert!(!res.attributes.is_empty());

    let msg = to_binary(&GovecExecuteMsg::Mint {
        new_wallet: wallet_addr.to_string(),
    })
    .unwrap();

    let msg = WasmMsg::Execute {
        contract_addr: GOVEC_ADDR.to_string(),
        msg,
        funds: vec![],
    };

    let reply_msg = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(to_binary(&msg).unwrap()),
        }),
    };

    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
    let ack: StdAck = from_binary(&res.data.unwrap()).unwrap();
    let ack: u64 = from_binary(&ack.unwrap()).unwrap();

    assert_eq!(ack, VectisDaoActionIds::GovecMint as u64)
}
