pub use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
pub use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Api, Attribute, BankMsg, Binary, Coin, CosmosMsg,
    DepsMut, Ibc3ChannelOpenResponse, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcMsg, IbcOrder,
    OwnedDeps, Reply, StdError, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};

pub use vectis_wallet::{
    ChainConfig, DaoConfig, DaoTunnelPacketMsg, IbcError, PacketMsg, StdAck,
    WalletFactoryExecuteMsg, WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER,
    IBC_APP_VERSION, PACKET_LIFETIME,
};

pub use crate::contract::{execute_dispatch, execute_mint_govec, instantiate, query, reply};
pub use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
pub use crate::msg::{IbcTransferChannels, InstantiateMsg, QueryMsg};
pub use crate::state::{CHAIN_CONFIG, DAO_CONFIG, JOB_ID};
pub use crate::{ContractError, FACTORY_CALLBACK_ID};

pub const INVALID_PORT_ID: &str = "wasm.invalid";
pub const DENOM: &str = "denom";
pub const DAO_CONNECTION_ID: &str = "connection-1";
pub const DAO_PORT_ID: &str = "wasm.dao";
pub const DAO_ADDR: &str = "wasm.address_dao";
pub const DAO_CHANNEL_ID: &str = "channel-1";
pub const OTHER_CONNECTION_ID: &str = "connection-1";
pub const OTHER_TRANSFER_PORT_ID: &str = "wasm.ibc_module";
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
    let chain_config = ChainConfig {
        remote_factory: None,
        demon: "cosm".to_string(),
    };

    let instantiate_msg = InstantiateMsg {
        dao_config,
        chain_config,
        denom: DENOM.to_string(),
        init_ibc_transfer_mod: Some(IbcTransferChannels {
            endpoints: vec![(
                OTHER_CONNECTION_ID.to_string(),
                OTHER_TRANSFER_PORT_ID.to_string(),
                None,
            )],
        }),
    };

    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    assert_eq!(res.attributes[0].key, "vectis-remote-tunnel instantiated");

    deps
}

//
// #[test]
// fn handle_dispatch_packet() {
//     let mut deps = do_instantiate();
//     let channel_id = "channel-123";
//
//     let dispatch_msg = CosmosMsg::Bank(BankMsg::Send {
//         to_address: "test".to_string(),
//         amount: vec![],
//     });
//
//     let msgs = vec![dispatch_msg];
//
//     let ibc_msg = PacketMsg::Dispatch {
//         msgs: msgs.clone(),
//         sender: "sender".to_string(),
//         job_id: Some("my_job_id".to_string()),
//     };
//
//     // register the channel
//     connect(deps.as_mut(), channel_id);
//
//     let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
//     let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
//
//     // assert app-level success
//     let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
//     ack.unwrap();
//
//     assert_eq!(msgs.len(), res.messages.len());
//
//     assert_eq!(RECEIVE_DISPATCH_ID, res.messages[0].id.clone());
//
//     let data = "string";
//
//     let mut encoded = vec![0x0a, data.len() as u8];
//     encoded.extend(data.as_bytes());
//
//     let msg = WasmMsg::Execute {
//         contract_addr: "address".to_string(),
//         msg: to_binary(&()).unwrap(),
//         funds: vec![],
//     };
//
//     let response = Reply {
//         id: res.messages[0].id,
//         result: SubMsgResult::Ok(SubMsgResponse {
//             events: vec![],
//             data: Some(to_binary(&msg).unwrap()),
//         }),
//     };
//
//     let res = reply(deps.as_mut(), mock_env(), response).unwrap();
//     let ack: StdAck = from_binary(&res.data.unwrap()).unwrap();
//     let response: DispatchResponse = from_binary(&ack.unwrap()).unwrap();
//
//     let result: WasmMsg = from_binary(&response.results[0]).unwrap();
//
//     let state = RESULTS.load(&deps.storage).unwrap();
//     let storage_result: WasmMsg = from_binary(&state[0]).unwrap();
//
//     assert_eq!(storage_result, result);
// }
//
// #[test]
// fn handle_update_packet() {
//     let mut deps = do_instantiate();
//     let channel_id = "channel-123";
//
//     let ibc_msg = PacketMsg::UpdateChannel;
//
//     connect(deps.as_mut(), channel_id);
//
//     let res: Option<String> =
//         from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Channel {}).unwrap()).unwrap();
//
//     assert!(res.is_none());
//
//     let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
//     let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
//
//     // assert app-level success
//     let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
//     ack.unwrap();
//
//     let res: Option<String> =
//         from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Channel {}).unwrap()).unwrap();
//
//     assert_eq!(channel_id.to_string(), res.unwrap());
// }
//
// #[test]
// fn handle_mint_govec_packet() {
//     let mut deps = do_instantiate();
//     let channel_id = "channel-123";
//     let factory_addr = "fake_addr";
//     let wallet_addr = "user_address";
//
//     let mut encoded = vec![0x0a, factory_addr.len() as u8];
//     encoded.extend(factory_addr.as_bytes());
//
//     let response = Reply {
//         id: FACTORY_CALLBACK_ID,
//         result: SubMsgResult::Ok(SubMsgResponse {
//             events: vec![],
//             data: Some(Binary::from(encoded)),
//         }),
//     };
//
//     reply(deps.as_mut(), mock_env(), response).unwrap();
//
//     let ibc_msg = PacketMsg::MintGovec {
//         wallet_addr: wallet_addr.to_string(),
//     };
//
//     connect(deps.as_mut(), channel_id);
//
//     let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
//     let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
//
//     // assert app-level success
//     let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
//     ack.unwrap();
//
//     assert_eq!(
//         CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: factory_addr.to_string(),
//             msg: to_binary(&WalletFactoryExecuteMsg::GovecMinted {
//                 wallet: wallet_addr.to_string(),
//             })
//             .unwrap(),
//             funds: vec![],
//         }),
//         res.messages[0].msg
//     )
// }
//
// #[test]
// fn handle_execute_mint_govec() {
//     let mut deps = do_instantiate();
//     let factory_addr = "factory_addr";
//     let cannonical_fact_addr = deps.as_mut().api.addr_canonicalize(factory_addr).unwrap();
//     let wallet_addr = "wallet_addr";
//     let info = mock_info("sender", &vec![]);
//     let env = mock_env();
//
//     DAO_TUNNEL_CHANNEL
//         .save(deps.as_mut().storage, &CHANNEL_ID.to_string())
//         .unwrap();
//
//     // should expect err because there is not factory in the state
//     let res = execute_mint_govec(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         wallet_addr.to_string(),
//     )
//     .unwrap_err();
//     assert_eq!(res, ContractError::NotFound("Factory".to_string()));
//
//     FACTORY
//         .save(deps.as_mut().storage, &cannonical_fact_addr)
//         .unwrap();
//
//     // should expect err because factory addr is different
//     let res = execute_mint_govec(
//         deps.as_mut(),
//         env.clone(),
//         info.clone(),
//         wallet_addr.to_string(),
//     )
//     .unwrap_err();
//     assert_eq!(res, ContractError::Unauthorized);
//
//     let res = execute_mint_govec(
//         deps.as_mut(),
//         env.clone(),
//         mock_info(factory_addr, &vec![]),
//         wallet_addr.to_string(),
//     )
//     .unwrap();
//
//     let packet = PacketMsg::MintGovec {
//         wallet_addr: wallet_addr.to_string(),
//     };
//
//     let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
//         channel_id: CHANNEL_ID.to_string(),
//         data: to_binary(&packet).unwrap(),
//         timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
//     });
//
//     assert_eq!(res.messages[0].msg, msg);
// }
//
// #[test]
// fn handle_execute_dispatch() {
//     let mut deps = do_instantiate();
//     let info = mock_info("sender", &vec![]);
//     let env = mock_env();
//
//     DAO_TUNNEL_CHANNEL
//         .save(deps.as_mut().storage, &CHANNEL_ID.to_string())
//         .unwrap();
//
//     let res = execute_dispatch(deps.as_mut(), env.clone(), info.clone(), vec![], None).unwrap();
//
//     let packet = PacketMsg::Dispatch {
//         sender: info.sender.to_string(),
//         job_id: None,
//         msgs: vec![],
//     };
//
//     let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
//         channel_id: CHANNEL_ID.to_string(),
//         data: to_binary(&packet).unwrap(),
//         timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
//     });
//
//     assert_eq!(res.messages[0].msg, msg);
// }
