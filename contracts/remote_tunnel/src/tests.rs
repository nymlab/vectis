use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, DepsMut,
    IbcChannelOpenMsg, IbcMsg, IbcOrder, OwnedDeps, Reply, SubMsgResponse, SubMsgResult, Uint128,
    WasmMsg,
};

use vectis_wallet::{
    DispatchResponse, IbcError, PacketMsg, StdAck,
    WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER, IBC_APP_VERSION,
    PACKET_LIFETIME, RECEIVE_DISPATCH_ID,
};

use crate::contract::{execute_dispatch, execute_mint_govec, instantiate, query, reply};
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::{CHANNEL, FACTORY, RESULTS};
use crate::{ContractError, FACTORY_CALLBACK_ID};

const CONNECTION_ID: &str = "connection-1";
const CHANNEL_ID: &str = "channel-1";
const PORT_ID: &str = "wasm.address";

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
    let connection_id = CONNECTION_ID.to_string();
    let port_id = PORT_ID.to_string();

    let mut deps = mock_dependencies();
    let info = mock_info("address", &[]);
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        connection_id,
        port_id,
    };

    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    assert_eq!(res.attributes[0].key, "vectis-remote-tunnel instantiated");

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

#[test]
fn cannot_handshake_with_wrong_config() {
    let mut deps = do_instantiate();

    // Wrong order
    let handshake_open = mock_ibc_channel_open_init(
        CONNECTION_ID,
        PORT_ID,
        CHANNEL_ID,
        IbcOrder::Ordered,
        IBC_APP_VERSION,
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidChannelOrder;

    assert_eq!(res, ContractError::IbcError(err_msg));

    // Wrong version
    let handshake_open = mock_ibc_channel_open_init(
        CONNECTION_ID,
        PORT_ID,
        CHANNEL_ID,
        APP_ORDER,
        "wrong-version",
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidChannelVersion(IBC_APP_VERSION);

    assert_eq!(res, ContractError::IbcError(err_msg));

    // Wrong connection_id
    let handshake_open = mock_ibc_channel_open_init(
        "wrong-connection",
        PORT_ID,
        CHANNEL_ID,
        APP_ORDER,
        IBC_APP_VERSION,
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidConnectionId(CONNECTION_ID.to_string());

    assert_eq!(res, ContractError::IbcError(err_msg));

    // Wrong port_id
    let handshake_open = mock_ibc_channel_open_init(
        CONNECTION_ID,
        "wrong-port",
        CHANNEL_ID,
        APP_ORDER,
        IBC_APP_VERSION,
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidPortId(PORT_ID.to_string());

    assert_eq!(res, ContractError::IbcError(err_msg));
}

#[test]
fn handle_factory_packet() {
    let mut deps = do_instantiate();
    let channel_id = "channel-123";

    // invalid packet format on registered channel also returns error
    let msg = mock_ibc_packet_recv(channel_id, &()).unwrap();
    ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap_err();

    let msg = FactoryInstantiateMsg {
        proxy_multisig_code_id: 13,
        addr_prefix: "prefix".to_string(),
        govec_minter: None,
        proxy_code_id: 13,
        wallet_fee: Coin {
            amount: Uint128::one(),
            denom: "denom".to_string(),
        },
    };

    let ibc_msg = PacketMsg::InstantiateFactory { code_id: 13, msg };

    connect(deps.as_mut(), channel_id);

    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    // assert app-level success
    let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
    ack.unwrap();

    assert_eq!(1, res.messages.len());

    assert_eq!(FACTORY_CALLBACK_ID, res.messages[0].id.clone());

    // ock factory response
    let contract_address = "fake_addr";

    let mut encoded = vec![0x0a, contract_address.len() as u8];
    encoded.extend(contract_address.as_bytes());

    let response = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(Binary::from(encoded)),
        }),
    };

    reply(deps.as_mut(), mock_env(), response).unwrap();
    let res: Option<Addr> =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Factory).unwrap()).unwrap();

    assert_eq!(contract_address.to_string(), res.unwrap());
}

#[test]
fn handle_dispatch_packet() {
    let mut deps = do_instantiate();
    let channel_id = "channel-123";

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

    // register the channel
    connect(deps.as_mut(), channel_id);

    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
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
fn handle_update_packet() {
    let mut deps = do_instantiate();
    let channel_id = "channel-123";

    let ibc_msg = PacketMsg::UpdateChannel;

    connect(deps.as_mut(), channel_id);

    let res: Option<String> =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Channel).unwrap()).unwrap();

    assert!(res.is_none());

    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    // assert app-level success
    let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
    ack.unwrap();

    let res: Option<String> =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Channel).unwrap()).unwrap();

    assert_eq!(channel_id.to_string(), res.unwrap());
}

#[test]
fn handle_mint_govec_packet() {
    let mut deps = do_instantiate();
    let channel_id = "channel-123";
    let factory_addr = "fake_addr";
    let wallet_addr = "user_address";

    let mut encoded = vec![0x0a, factory_addr.len() as u8];
    encoded.extend(factory_addr.as_bytes());

    let response = Reply {
        id: FACTORY_CALLBACK_ID,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(Binary::from(encoded)),
        }),
    };

    reply(deps.as_mut(), mock_env(), response).unwrap();

    let ibc_msg = PacketMsg::MintGovec {
        wallet_addr: wallet_addr.to_string(),
    };

    connect(deps.as_mut(), channel_id);

    let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    // assert app-level success
    let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
    ack.unwrap();

    assert_eq!(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: factory_addr.to_string(),
            msg: Binary::from(vec![]),
            funds: vec![],
        }),
        res.messages[0].msg
    )
}

#[test]
fn handle_execute_mint_govec() {
    let mut deps = do_instantiate();
    let factory_addr = "factory_addr";
    let cannonical_fact_addr = deps.as_mut().api.addr_canonicalize(factory_addr).unwrap();
    let wallet_addr = "wallet_addr";
    let info = mock_info("sender", &vec![]);
    let env = mock_env();

    CHANNEL
        .save(deps.as_mut().storage, &CHANNEL_ID.to_string())
        .unwrap();

    // should expect err because there is not factory in the state
    let res = execute_mint_govec(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        wallet_addr.to_string(),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::NotFound("Factory".to_string()));

    FACTORY
        .save(deps.as_mut().storage, &cannonical_fact_addr)
        .unwrap();

    // should expect err because factory addr is different
    let res = execute_mint_govec(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        wallet_addr.to_string(),
    )
    .unwrap_err();
    assert_eq!(res, ContractError::Unauthorized);

    let res = execute_mint_govec(
        deps.as_mut(),
        env.clone(),
        mock_info(factory_addr, &vec![]),
        wallet_addr.to_string(),
    )
    .unwrap();

    let packet = PacketMsg::MintGovec {
        wallet_addr: wallet_addr.to_string(),
    };

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&packet).unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
}

#[test]
fn handle_execute_dispatch() {
    let mut deps = do_instantiate();
    let info = mock_info("sender", &vec![]);
    let env = mock_env();

    CHANNEL
        .save(deps.as_mut().storage, &CHANNEL_ID.to_string())
        .unwrap();

    let res = execute_dispatch(deps.as_mut(), env.clone(), info.clone(), vec![], None).unwrap();

    let packet = PacketMsg::Dispatch {
        sender: info.sender.to_string(),
        job_id: None,
        msgs: vec![],
    };

    let msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
        channel_id: CHANNEL_ID.to_string(),
        data: to_binary(&packet).unwrap(),
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    });

    assert_eq!(res.messages[0].msg, msg);
}
