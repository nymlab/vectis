pub use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_close_init,
    mock_ibc_channel_connect_ack, mock_ibc_packet_ack, mock_ibc_packet_recv, mock_info, MockApi,
    MockQuerier, MockStorage,
};
pub use cosmwasm_std::{
    from_binary, to_binary, Attribute, Binary, CanonicalAddr, Coin, CosmosMsg, DepsMut, Empty,
    Ibc3ChannelOpenResponse, IbcAcknowledgement, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg,
    IbcOrder, OwnedDeps, Reply, StdError, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};

pub use vectis_wallet::{
    ChainConfig, DaoTunnelPacketMsg, GovecExecuteMsg, IbcError, PacketMsg, ProposalExecuteMsg,
    RemoteTunnelPacketMsg, StdAck, VectisDaoActionIds,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER, IBC_APP_VERSION,
    PACKET_LIFETIME,
};
pub use voting::Vote;

pub use crate::contract::{execute, instantiate, query_controllers, query_dao, query_govec, reply};
pub use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_ack, ibc_packet_receive};
pub use crate::msg::{ExecuteMsg, InstantiateMsg, RemoteTunnels};
pub use crate::state::ADMIN;
pub use crate::ContractError;

pub const TEST_CONNECTION_ID: &str = "TEST_CONNECTION_ID";
pub const CHANNEL_ID: &str = "channel-1";
// To match src port id in `mock_ibc_packet_recv`
// For testing `ibc_packet_receive`_
pub const SRC_PORT_ID_RCV: &str = "their-port";
// To match src port id in `mock_ibc_channel`
// For testing `ibc_channel_connect`
pub const SRC_PORT_ID_CONNECT: &str = "their_port";

pub const ADMIN_ADDR: &str = "admin_addr";
pub const GOVEC_ADDR: &str = "govec_addr";
pub const JOB_ID: u64 = 8721;
pub const WALLET_ADDR: &str = "proxy_wallet_addr";
pub const FACTORY_ADDR: &str = "factory_addr";
pub const DENOM: &str = "denom";

pub fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN_ADDR, &[]);
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        govec_minter: GOVEC_ADDR.to_string(),
        init_remote_tunnels: None,
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

// Tests for contract ExecuteMsg functions
#[test]
fn not_admin_approve_contollers_fails() {
    let mut deps = do_instantiate();

    // not admin fails
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
}

#[test]
fn admin_can_update_approved_contollers() {
    let mut deps = do_instantiate();

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

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: "ANOTHER".to_string(),
            port_id: "ANOTHER_PORT".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            ("action", "add_approved_controller"),
            ("connection_id", "ANOTHER"),
            ("port_id", "ANOTHER_PORT")
        ]
    );

    // Pagination query
    let res = query_controllers(
        deps.as_ref(),
        Some(("ANOTHER".to_string(), "ANOTHER_PORT".to_string())),
        None,
    )
    .unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(
                TEST_CONNECTION_ID.to_string(),
                SRC_PORT_ID_CONNECT.to_string()
            )]
        },
        res
    );

    // Remove controller
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::RemoveApprovedController {
            connection_id: "ANOTHER".to_string(),
            port_id: "ANOTHER_PORT".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![
            ("action", "remove_approved_controller"),
            ("connection_id", "ANOTHER"),
            ("port_id", "ANOTHER_PORT")
        ]
    );

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
fn only_admin_can_update_govec() {
    let mut deps = do_instantiate();
    // Not admin fails
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("RANDOM", &[]),
        ExecuteMsg::UpdateGovecAddr {
            new_addr: "new_govec".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Update Govec
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::UpdateGovecAddr {
            new_addr: "new_govec".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![("config", "GOVEC Addr"), ("new addr", "new_govec")]
    );

    let res = query_govec(deps.as_ref()).unwrap().unwrap();
    assert_eq!(res.as_str(), "new_govec");
}

#[test]
fn only_admin_can_update_selfaddr() {
    let mut deps = do_instantiate();
    // Not admin fails
    let err = execute(
        deps.as_mut(),
        mock_env(),
        mock_info("RANDOM", &[]),
        ExecuteMsg::UpdateDaoAddr {
            new_addr: "new_govec".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Update Govec
    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::UpdateDaoAddr {
            new_addr: "new_dao".to_string(),
        },
    )
    .unwrap();

    assert_eq!(
        res.attributes,
        vec![("config", "DAO Addr"), ("new addr", "new_dao")]
    );

    let res = query_dao(deps.as_ref()).unwrap().unwrap();
    assert_eq!(res.as_str(), "new_dao");
}

fn default_update_chain_config_msg() -> DaoTunnelPacketMsg {
    let mut encoded_factory = vec![0x0a, FACTORY_ADDR.len() as u8];
    encoded_factory.extend(FACTORY_ADDR.as_bytes());

    DaoTunnelPacketMsg::UpdateChainConfig {
        new_config: ChainConfig {
            remote_factory: Some(CanonicalAddr::from(encoded_factory)),
            demon: DENOM.to_string(),
        },
    }
}

#[test]
fn only_admin_can_send_actions_to_remote_tunnel_channel() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let invalid_sender = "RANDOM_ADDR";
    let valid_sender = ADMIN_ADDR;
    let chain_update_msg = default_update_chain_config_msg();

    // Not DAO cannot send this
    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(invalid_sender, &[]),
        ExecuteMsg::DispatchActionOnRemoteTunnel {
            job_id: JOB_ID,
            msg: chain_update_msg.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized);

    // Valid sender
    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(valid_sender, &[]),
        ExecuteMsg::DispatchActionOnRemoteTunnel {
            job_id: JOB_ID,
            msg: chain_update_msg.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap();

    let expected_msg: CosmosMsg<Empty> = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&PacketMsg {
            // This should be the contract address
            sender: env.contract.address.to_string(),
            job_id: JOB_ID,
            msg: to_binary(&chain_update_msg).unwrap(),
        })
        .unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, expected_msg);
    assert_eq!(
        res.attributes,
        vec![
            ("action", "dispatch_actions_to_remote_tunnel"),
            ("channel", CHANNEL_ID),
            ("job_id", &JOB_ID.to_string())
        ]
    )
}
