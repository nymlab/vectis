use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_channel_connect_ack,
    mock_ibc_packet_recv, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    from_binary, from_slice, Addr, Binary, Coin, DepsMut, IbcChannelOpenMsg, IbcOrder, OwnedDeps,
    Reply, SubMsgResponse, SubMsgResult, Uint128, CosmosMsg, BankMsg, StdResult,
};

use vectis_wallet::{
    IbcError, PacketMsg, StdAck, WalletFactoryInstantiateMsg as FactoryInstantiateMsg, APP_ORDER,
    IBC_APP_VERSION, RECEIVE_DISPATCH_ID
};

use crate::contract::{instantiate, query, reply};
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ibc_packet_receive};
use crate::msg::{InstantiateMsg, QueryMsg};
use crate::state::RESULTS;
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
        govec: None,
        proxy_code_id: 13,
        wallet_fee: Coin {
            amount: Uint128::one(),
            denom: "denom".to_string(),
        },
    };

    let ibc_msg = PacketMsg::InstantiateFactory { code_id: 13, msg };

    // register the channel
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

    // let info = mock_info("sender", &vec![]);

    let dispatch_msg = CosmosMsg::Bank(BankMsg::Send { to_address: "test".to_string(), amount: vec![] });

    let msgs = vec![dispatch_msg];

    let ibc_msg = PacketMsg::Dispatch { msgs: msgs.clone(), sender: "sender".to_string(), job_id: Some("my_job_id".to_string()) };

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

    let response = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(Binary::from(encoded.clone())),
        }),
    };

    let res = reply(deps.as_mut(), mock_env(), response).unwrap();
    let binary = res.data.unwrap();
    
    // assert_eq!(contract_address.to_string(), res.unwrap());
}