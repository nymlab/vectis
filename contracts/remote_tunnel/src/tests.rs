use cosmwasm_std::coin;
pub use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_close_confirm,
    mock_ibc_channel_connect_ack, mock_ibc_packet_ack, mock_ibc_packet_recv, mock_info, MockApi,
    MockQuerier, MockStorage,
};
pub use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    DepsMut, Ibc3ChannelOpenResponse, IbcAcknowledgement, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcMsg, IbcOrder, OwnedDeps, Reply, StdError, SubMsg, SubMsgResponse,
    SubMsgResult, Uint128, WasmMsg,
};

use vectis_wallet::DaoActors;
pub use vectis_wallet::{
    DaoConfig, DaoTunnelPacketMsg, IbcError, PacketMsg, RemoteTunnelPacketMsg, StdAck,
    VectisDaoActionIds, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, IBC_APP_ORDER, IBC_APP_VERSION, ITEMS,
    PACKET_LIFETIME,
};

pub use crate::contract::{execute_dispatch, execute_mint_govec, instantiate, query, reply};
use crate::contract::{
    execute_ibc_transfer, query_channels, query_dao_config, query_item, query_job_id,
};
pub use crate::ibc::{
    ibc_channel_close, ibc_channel_connect, ibc_channel_open, ibc_packet_ack, ibc_packet_receive,
};
pub use crate::msg::{IbcTransferChannels, InstantiateMsg, QueryMsg};
pub use crate::state::{DAO_CONFIG, JOB_ID};
use crate::tests_ibc::connect;
pub use crate::{ContractError, DISPATCH_CALLBACK_ID, FACTORY_CALLBACK_ID};

pub const INVALID_PORT_ID: &str = "wasm.invalid";
pub const DENOM: &str = "denom";
pub const DAO_CONNECTION_ID: &str = "connection-1";
pub const DAO_PORT_ID: &str = "wasm.dao";
pub const DAO_ADDR: &str = "wasm.address_dao";
pub const DAO_CHANNEL_ID: &str = "channel-1";
pub const OTHER_CONNECTION_ID: &str = "connection-1";
pub const OTHER_CHANNEL_ID: &str = "channel-2";

pub fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info("address", &[]);
    let env = mock_env();
    let dao_config = DaoConfig {
        addr: DAO_ADDR.to_string(),
        dao_tunnel_port_id: DAO_PORT_ID.to_string(),
        connection_id: DAO_CONNECTION_ID.to_string(),
        dao_tunnel_channel: None,
    };
    let instantiate_msg = InstantiateMsg {
        dao_config,
        init_ibc_transfer_mod: Some(IbcTransferChannels {
            endpoints: vec![
                (
                    OTHER_CONNECTION_ID.to_string(),
                    OTHER_CHANNEL_ID.to_string(),
                ),
                ("connection-one".to_string(), "chan-one".to_string()),
                ("connection-two".to_string(), "chan-two".to_string()),
            ],
        }),
        init_items: Some(vec![(
            DaoActors::Factory.to_string(),
            DaoActors::Factory.to_string(),
        )]),
    };

    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    assert_eq!(res.attributes[0].key, "vectis-remote-tunnel instantiated");

    deps
}

#[test]
fn queries_works() {
    let deps = do_instantiate();
    let expected_dao_config = DaoConfig {
        addr: DAO_ADDR.to_string(),
        dao_tunnel_port_id: DAO_PORT_ID.to_string(),
        connection_id: DAO_CONNECTION_ID.to_string(),
        dao_tunnel_channel: None,
    };

    let dao_config = query_dao_config(deps.as_ref()).unwrap();
    let next_job_id = query_job_id(deps.as_ref()).unwrap();
    let factory = query_item(deps.as_ref(), DaoActors::Factory.to_string()).unwrap();

    let all_tunnels = query_channels(deps.as_ref(), None, None).unwrap();
    let last_tunnel =
        query_channels(deps.as_ref(), Some("connection-one".to_string()), None).unwrap();

    assert_eq!(expected_dao_config, dao_config);
    assert_eq!(factory, DaoActors::Factory.to_string());
    assert_eq!(next_job_id, 0);
    assert_eq!(
        all_tunnels,
        IbcTransferChannels {
            endpoints: vec![
                ("connection-two".to_string(), "chan-two".to_string()),
                ("connection-one".to_string(), "chan-one".to_string()),
                (
                    OTHER_CONNECTION_ID.to_string(),
                    OTHER_CHANNEL_ID.to_string(),
                ),
            ]
        }
    );
    assert_eq!(
        last_tunnel,
        IbcTransferChannels {
            endpoints: vec![("connection-two".to_string(), "chan-two".to_string()),]
        }
    );
}

#[test]
fn test_mint_govec_must_be_from_factory() {
    let mut deps = do_instantiate();
    let err = execute_dispatch(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: "Someaddr".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);
    ITEMS.remove(deps.as_mut().storage, DaoActors::Factory.to_string());
    execute_dispatch(
        deps.as_mut(),
        mock_env(),
        mock_info(DaoActors::Factory.to_string().as_str(), &[]),
        RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: "Someaddr".to_string(),
        },
    )
    .unwrap_err();
}

#[test]
fn test_dao_action_fails_without_dao_channel() {
    let mut deps = do_instantiate();
    let err = execute_dispatch(
        deps.as_mut(),
        mock_env(),
        mock_info("sender", &[]),
        RemoteTunnelPacketMsg::StakeActions(vectis_wallet::StakeExecuteMsg::Claim {
            relayed_from: None,
        }),
    )
    .unwrap_err();

    assert_eq!(err, ContractError::DaoChannelNotFound);
}

#[test]
fn ibc_transfer_fails_without_channel() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let err = execute_ibc_transfer(
        deps.as_mut(),
        env,
        mock_info("sender", &[coin(11u128, DENOM), coin(22u128, DENOM)]),
        crate::msg::Receiver {
            connection_id: "NOT_VALID_CONNECTION".to_string(),
            addr: "receiver".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        err,
        ContractError::ChannelNotFound("NOT_VALID_CONNECTION".to_string())
    )
}

#[test]
fn ibc_transfer_fails_without_funds() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let err = execute_ibc_transfer(
        deps.as_mut(),
        env.clone(),
        mock_info("sender", &[]),
        crate::msg::Receiver {
            connection_id: OTHER_CONNECTION_ID.to_string(),
            addr: "receiver".to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::EmptyFund);
}

#[test]
fn dao_actions_works_with_connected_channel() {
    // Tests that the message is fired and correct events are emitted
    let mut deps = do_instantiate();
    let env = mock_env();
    connect(deps.as_mut(), "dao_channel_id");
    let job_id = query_job_id(deps.as_ref()).unwrap();
    let msg = RemoteTunnelPacketMsg::StakeActions(vectis_wallet::StakeExecuteMsg::Claim {
        relayed_from: None,
    });
    let res = execute_dispatch(
        deps.as_mut(),
        env.clone(),
        mock_info("sender", &[]),
        msg.clone(),
    )
    .unwrap();

    let packet = PacketMsg {
        sender: "sender".to_string(),
        job_id,
        msg: to_binary(&msg).unwrap(),
    };
    let msg = IbcMsg::SendPacket {
        channel_id: "dao_channel_id".to_string(),
        data: to_binary(&packet).unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };
    assert_eq!(res.messages[0], SubMsg::new(msg));

    assert_eq!(
        res.events[0].attributes,
        vec![("job_id", &job_id.to_string())]
    );

    let next_job_id = query_job_id(deps.as_ref()).unwrap();
    assert_eq!(next_job_id, job_id + 1);
}

#[test]
fn ibc_transfer_works_with_channel_connected() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let coin = coin(11u128, DENOM);
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let res = execute_ibc_transfer(
        deps.as_mut(),
        env.clone(),
        mock_info("sender", &[coin.clone()]),
        crate::msg::Receiver {
            connection_id: OTHER_CONNECTION_ID.to_string(),
            addr: "receiver".to_string(),
        },
    )
    .unwrap();

    let msg = IbcMsg::Transfer {
        channel_id: OTHER_CHANNEL_ID.to_string(),
        to_address: "receiver".to_string(),
        amount: coin.clone(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };
    assert_eq!(res.messages[0], SubMsg::new(msg));

    assert_eq!(
        res.events[0].attributes,
        vec![
            ("channel_id", OTHER_CHANNEL_ID),
            ("to", "receiver"),
            ("amount", &coin.amount.to_string()),
            ("denom", &coin.denom.to_string()),
        ]
    )
}
