use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, BankMsg, Coin, CosmosMsg, DepsMut, IbcChannelOpenMsg,
    IbcMsg, IbcOrder, OwnedDeps, Reply, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};

use vectis_govec::msg::ExecuteMsg as GovecExecuteMsg;
use vectis_wallet::{
    DispatchResponse, IbcError, PacketMsg, StdAck,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER, IBC_APP_VERSION,
    PACKET_LIFETIME, RECEIVE_DISPATCH_ID,
};

use crate::contract::{execute, instantiate, query_controllers, query_govec, reply};
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
use crate::msg::{ExecuteMsg, InstantiateMsg, RemoteTunnels};
use crate::state::{ADMIN, RESULTS};
use crate::ContractError;

const CONNECTION_ID: &str = "connection-1";
const CHANNEL_ID: &str = "channel-1";
const PORT_ID: &str = "wasm.address";
const ADMIN_ADDR: &str = "admin_addr";
const GOVEC_ADDR: &str = "govec_addr";

fn mock_ibc_channel_open_init(
    connection_id: &str,
    port_id: &str,
    channel_id: &str,
    order: IbcOrder,
    version: &str,
) -> IbcChannelOpenMsg {
    let mut channel = mock_ibc_channel(channel_id, order, version);
    channel.connection_id = connection_id.to_string();
    channel.endpoint.port_id = port_id.to_string();
    IbcChannelOpenMsg::new_init(channel)
}

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

fn connect(mut deps: DepsMut, channel_id: &str) {
    let handshake_open = mock_ibc_channel_open_init(
        CONNECTION_ID,
        PORT_ID,
        channel_id,
        APP_ORDER,
        IBC_APP_VERSION,
    );

    ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

    let handshake_connect = mock_ibc_channel_connect_ack(channel_id, APP_ORDER, IBC_APP_VERSION);

    let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();

    assert_eq!(
        res.attributes,
        vec![("action", "ibc_connect"), ("channel_id", channel_id)]
    );
}

fn add_mock_controller(mut deps: DepsMut) {
    execute(
        deps.branch(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: CONNECTION_ID.to_string(),
            port_id: PORT_ID.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(CONNECTION_ID.to_string(), PORT_ID.to_string())]
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
            connection_id: CONNECTION_ID.to_string(),
            port_id: PORT_ID.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);

    execute(
        deps.as_mut(),
        mock_env(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::AddApprovedController {
            connection_id: CONNECTION_ID.to_string(),
            port_id: PORT_ID.to_string(),
        },
    )
    .unwrap();

    let res = query_controllers(deps.as_ref(), None, None).unwrap();

    assert_eq!(
        RemoteTunnels {
            tunnels: vec![(CONNECTION_ID.to_string(), PORT_ID.to_string())]
        },
        res
    );
}

#[test]
fn only_admin_can_update_remote_tunnel_channel() {
    let mut deps = do_instantiate();
    let env = mock_env();

    let channel_id = "new_channel_id";

    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("RANDOM_ADDR", &[]),
        ExecuteMsg::UpdateRemoteTunnelChannel {
            channel_id: channel_id.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::UpdateRemoteTunnelChannel {
            channel_id: channel_id.to_string(),
        },
    )
    .unwrap();

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: channel_id.to_string(),
        data: to_binary(&PacketMsg::UpdateChannel).unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
    assert_eq!(
        res.attributes,
        vec![
            ("action", "update_remote_tunnel_channel"),
            ("channel_id", channel_id)
        ]
    )
}

#[test]
fn only_admin_can_instantiate_factory() {
    let mut deps = do_instantiate();
    let env = mock_env();

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
        mock_info("RANDOM_ADDR", &[]),
        ExecuteMsg::InstantiateRemoteFactory {
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
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::InstantiateRemoteFactory {
            code_id: 45,
            msg: instantiation_msg.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap();

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&PacketMsg::InstantiateFactory {
            code_id: 45,
            msg: instantiation_msg,
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

#[test]
fn only_admin_can_dispatch() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let job_id = Some("23".to_string());
    let mock_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: "address".to_string(),
        amount: vec![Coin {
            amount: Uint128::MAX,
            denom: "denom".to_string(),
        }],
    });

    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("RANDOM_ADDR", &[]),
        ExecuteMsg::Dispatch {
            msgs: vec![mock_msg.clone()],
            job_id: job_id.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized);

    let res = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::Dispatch {
            msgs: vec![mock_msg.clone()],
            job_id: job_id.clone(),
            channel_id: CHANNEL_ID.to_string(),
        },
    )
    .unwrap();

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&PacketMsg::Dispatch {
            msgs: vec![mock_msg],
            sender: ADMIN_ADDR.to_string(),
            job_id: job_id,
        })
        .unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
    assert_eq!(res.attributes, vec![("action", "execute_dispatch")])
}

#[test]
fn throw_error_when_invalid_ibc_packet() {
    let mut deps = do_instantiate();
    let ibc_msg = PacketMsg::UpdateChannel;

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let err = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap_err();

    assert_eq!(err, ContractError::IbcError(IbcError::InvalidPacket));
}

#[test]
fn handle_dispatch_packet() {
    let mut deps = do_instantiate();

    let dispatch_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: "test".to_string(),
        amount: vec![],
    });

    let msgs = vec![dispatch_msg];

    let ibc_msg = PacketMsg::Dispatch {
        msgs: msgs.clone(),
        sender: "sender".to_string(),
        job_id: Some("my_job_id".to_string()),
    };

    add_mock_controller(deps.as_mut());
    connect(deps.as_mut(), CHANNEL_ID);

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    // assert app-level success
    let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
    ack.unwrap();

    assert_eq!(msgs.len(), res.messages.len());

    assert_eq!(RECEIVE_DISPATCH_ID, res.messages[0].id.clone());

    let data = "string";

    let mut encoded = vec![0x0a, data.len() as u8];
    encoded.extend(data.as_bytes());

    let msg = WasmMsg::Execute {
        contract_addr: "address".to_string(),
        msg: to_binary(&()).unwrap(),
        funds: vec![],
    };

    let response = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(to_binary(&msg).unwrap()),
        }),
    };

    let res = reply(deps.as_mut(), mock_env(), response).unwrap();
    let ack: StdAck = from_binary(&res.data.unwrap()).unwrap();
    let response: DispatchResponse = from_binary(&ack.unwrap()).unwrap();

    let result: WasmMsg = from_binary(&response.results[0]).unwrap();

    let state = RESULTS.load(&deps.storage).unwrap();
    let storage_result: WasmMsg = from_binary(&state[0]).unwrap();

    assert_eq!(storage_result, result);
}

#[test]
fn handle_receive_dispatch_packet() {
    let mut deps = do_instantiate();
    let wallet_addr = "proxy_wallet_addr";
    let ibc_msg = PacketMsg::MintGovec {
        wallet_addr: wallet_addr.to_string(),
    };

    add_mock_controller(deps.as_mut());
    connect(deps.as_mut(), CHANNEL_ID);

    let msg = mock_ibc_packet_recv(CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
    let ack: bool = from_binary(&ack.unwrap()).unwrap();

    assert!(!ack);

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
    let ack: bool = from_binary(&ack.unwrap()).unwrap();

    assert!(ack)
}
