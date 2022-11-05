use crate::tests::*;

pub fn mock_ibc_channel_open_init(
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

pub fn mock_ibc_channel_open_try(
    connection_id: &str,
    port_id: &str,
    channel_id: &str,
    order: IbcOrder,
    counterparty_version: &str,
) -> IbcChannelOpenMsg {
    let mut channel = mock_ibc_channel(channel_id, order, counterparty_version);
    channel.connection_id = connection_id.to_string();
    channel.endpoint.port_id = port_id.to_string();
    IbcChannelOpenMsg::new_try(channel, counterparty_version)
}

// util to add `DAO_TUNNEL_CHANNEL` and `IBC_TRANSFER_PORT_ID`
pub fn connect(mut deps: DepsMut, dao_channel_id: &str, ibc_transfer_channel_id: &str) {
    if let IbcChannelConnectMsg::OpenAck {
        mut channel,
        counterparty_version,
    } = mock_ibc_channel_connect_ack(dao_channel_id, APP_ORDER, IBC_APP_VERSION)
    {
        // add dao tunnel
        channel.counterparty_endpoint.port_id = DAO_PORT_ID.to_string();
        channel.connection_id = DAO_CONNECTION_ID.to_string();
        let res = ibc_channel_connect(
            deps.branch(),
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel: channel.clone(),
                counterparty_version: counterparty_version.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "ibc_connect"),
                Attribute::new("SAVED local dao_tunnel channel_id", dao_channel_id),
                Attribute::new("dao_tunnel_port_id", DAO_PORT_ID)
            ]
        );

        // add ibc transfer
        channel.counterparty_endpoint.port_id = OTHER_TRANSFER_PORT_ID.to_string();
        channel.endpoint.channel_id = ibc_transfer_channel_id.to_string();
        channel.connection_id = OTHER_CONNECTION_ID.to_string();

        let res = ibc_channel_connect(
            deps,
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel: channel.clone(),
                counterparty_version: counterparty_version.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "ibc_connect"),
                Attribute::new("SAVED new ibc_transfer_channel_id", ibc_transfer_channel_id),
                Attribute::new("ibc_transfer_port_id", OTHER_TRANSFER_PORT_ID)
            ]
        );
    }
}

// Tests `ibc_channel_open`
#[test]
fn channel_open_only_right_version_order() {
    let mut deps = do_instantiate();
    // Wrong order
    let handshake_open = mock_ibc_channel_open_init(
        DAO_CONNECTION_ID,
        DAO_PORT_ID,
        DAO_CHANNEL_ID,
        IbcOrder::Ordered,
        IBC_APP_VERSION,
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidChannelOrder;

    assert_eq!(res, ContractError::IbcError(err_msg));

    // Wrong version
    let handshake_open = mock_ibc_channel_open_try(
        DAO_CONNECTION_ID,
        DAO_PORT_ID,
        DAO_CHANNEL_ID,
        APP_ORDER,
        "wrong-version",
    );
    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap_err();
    let err_msg = IbcError::InvalidChannelVersion(IBC_APP_VERSION);

    assert_eq!(res, ContractError::IbcError(err_msg));

    let handshake_open = mock_ibc_channel_open_init(
        DAO_CONNECTION_ID,
        DAO_PORT_ID,
        DAO_CHANNEL_ID,
        APP_ORDER,
        IBC_APP_VERSION,
    );

    let res = ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap();

    assert_eq!(
        res,
        Some(Ibc3ChannelOpenResponse {
            version: IBC_APP_VERSION.to_string()
        })
    );
}

// Tests `ibc_channel_connect`
#[test]
fn only_approved_endpoint_can_connect() {
    let mut deps = do_instantiate();

    if let IbcChannelConnectMsg::OpenAck {
        mut channel,
        counterparty_version,
    } = mock_ibc_channel_connect_ack(DAO_CHANNEL_ID, APP_ORDER, IBC_APP_VERSION)
    {
        // try to connect with port id not added as dao_tunnel or ibc_transfer
        channel.counterparty_endpoint.port_id = INVALID_PORT_ID.to_string();
        channel.connection_id = OTHER_CONNECTION_ID.to_string();
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
                msg: ContractError::Unauthorized.to_string()
            }
        );

        // try to connect with connection_id not added as dao_tunnel or ibc_transfer
        channel.counterparty_endpoint.port_id = DAO_PORT_ID.to_string();
        channel.connection_id = "INVALID_CONNECTION_ID".to_string();
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
                msg: ContractError::Unauthorized.to_string()
            }
        );

        // connect  port id is added to as dao_tunnel
        channel.counterparty_endpoint.port_id = DAO_PORT_ID.to_string();
        channel.connection_id = DAO_CONNECTION_ID.to_string();
        let res = ibc_channel_connect(
            deps.as_mut(),
            mock_env(),
            IbcChannelConnectMsg::OpenAck {
                channel: channel.clone(),
                counterparty_version: counterparty_version.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            res.attributes,
            vec![
                Attribute::new("action", "ibc_connect"),
                Attribute::new("SAVED local dao_tunnel channel_id", DAO_CHANNEL_ID),
                Attribute::new("dao_tunnel_port_id", DAO_PORT_ID)
            ]
        );

        // connect port id is added as IBC_TRANSFER
        channel.counterparty_endpoint.port_id = OTHER_TRANSFER_PORT_ID.to_string();
        channel.endpoint.channel_id = OTHER_CHANNEL_ID.to_string();
        channel.connection_id = OTHER_CONNECTION_ID.to_string();
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
                Attribute::new("SAVED new ibc_transfer_channel_id", OTHER_CHANNEL_ID),
                Attribute::new("ibc_transfer_port_id", OTHER_TRANSFER_PORT_ID)
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
    connect(deps.as_mut(), DAO_CHANNEL_ID, OTHER_CHANNEL_ID);

    let incorrect_ibc_msg = &[2; 12];

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &incorrect_ibc_msg).unwrap();
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
    connect(deps.as_mut(), DAO_CHANNEL_ID, OTHER_CHANNEL_ID);
    let incorrect_inner_msg = PacketMsg {
        sender: "Sender".to_string(),
        job_id: 1,
        // remote tunnel expects DaoTunnelPacketMsg
        msg: to_binary(&[2; 0]).unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &incorrect_inner_msg).unwrap();
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
fn handle_factory_packet() {
    let mut deps = do_instantiate();
    let job_id = 123;

    // This sets states so that ibc packet can be received
    connect(deps.as_mut(), DAO_CHANNEL_ID, OTHER_CHANNEL_ID);

    let factory_msg = FactoryInstantiateMsg {
        proxy_multisig_code_id: 19,
        addr_prefix: "prefix".to_string(),
        govec_minter: None,
        proxy_code_id: 13,
        wallet_fee: Coin {
            amount: Uint128::one(),
            denom: "denom".to_string(),
        },
    };

    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::InstantiateFactory {
            code_id: 123,
            msg: factory_msg,
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

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

    let res: ChainConfig =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::ChainConfig {}).unwrap()).unwrap();

    assert_eq!(
        contract_address.to_string(),
        deps.api
            .addr_humanize(&res.remote_factory.unwrap())
            .unwrap()
    );
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
