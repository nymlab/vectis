use crate::{contract::query_channels, tests::*};

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
pub fn connect(mut deps: DepsMut, dao_channel_id: &str) {
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
        channel.connection_id = DAO_CONNECTION_ID.to_string();
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
    } else {
        assert!(false)
    };
}

// Tests for `ibc_packet_receive`

#[test]
fn returns_ack_failure_when_invalid_ibc_packet_msg() {
    let mut deps = do_instantiate();
    connect(deps.as_mut(), DAO_CHANNEL_ID);

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
    connect(deps.as_mut(), DAO_CHANNEL_ID);
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
fn close_channels_reset_states() {
    let mut deps = do_instantiate();
    connect(deps.as_mut(), DAO_CHANNEL_ID);

    let init_msg = mock_ibc_channel_close_confirm(DAO_CHANNEL_ID, APP_ORDER, IBC_APP_VERSION);
    let mut channel = init_msg.channel().to_owned();
    channel.connection_id = DAO_CONNECTION_ID.to_string();
    channel.endpoint.channel_id = DAO_CHANNEL_ID.to_string();
    channel.counterparty_endpoint.port_id = DAO_PORT_ID.to_string();
    let msg = IbcChannelCloseMsg::new_confirm(channel);
    let res = ibc_channel_close(deps.as_mut(), mock_env(), msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("action", "ibc_close"),
            ("channel_id", DAO_CHANNEL_ID),
            ("src_port_id", DAO_PORT_ID),
            ("connection_id", DAO_CONNECTION_ID)
        ]
    )
}

#[test]
fn handle_inst_factory_packet() {
    let mut deps = do_instantiate();
    let job_id = 123;

    // This sets states so that ibc packet can be received
    connect(deps.as_mut(), DAO_CHANNEL_ID);

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

    // ack factory response
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

#[test]
fn handle_update_dao_config() {
    let mut deps = do_instantiate();
    let job_id = 123;
    connect(deps.as_mut(), DAO_CHANNEL_ID);

    let new_config = DaoConfig {
        addr: "new_dao".to_string(),
        dao_tunnel_port_id: "new_port".to_string(),
        connection_id: "new_connection".to_string(),
        dao_tunnel_channel: None,
    };
    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::UpdateDaoConfig {
            new_config: new_config.clone(),
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    let ack: StdAck = from_binary(&res.acknowledgement).unwrap();

    let actual_config = query(deps.as_ref(), mock_env(), QueryMsg::DaoConfig {}).unwrap();
    assert_eq!(new_config, from_binary(&actual_config).unwrap());

    assert_eq!(0, res.messages.len());
    assert_eq!(res.attributes, vec![("action", "dao config updated")]);
    assert_eq!(ack, StdAck::Result(to_binary(&job_id).unwrap()))
}

#[test]
fn handle_update_chain_config() {
    let mut deps = do_instantiate();
    let job_id = 123;
    connect(deps.as_mut(), DAO_CHANNEL_ID);

    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::UpdateChainConfig {
            new_config: ChainConfig {
                remote_factory: None,
                denom: "new_denom".to_string(),
            },
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    let ack: StdAck = from_binary(&res.acknowledgement).unwrap();

    let actual_config = query(deps.as_ref(), mock_env(), QueryMsg::ChainConfig {}).unwrap();
    assert_eq!(
        ChainConfig {
            remote_factory: None,
            denom: "new_denom".to_string(),
        },
        from_binary(&actual_config).unwrap()
    );

    assert_eq!(0, res.messages.len());
    assert_eq!(res.attributes, vec![("action", "chain config updated")]);
    assert_eq!(ack, StdAck::Result(to_binary(&job_id).unwrap()))
}

#[test]
fn handle_ibc_transfer_receiver() {
    let mut deps = do_instantiate();
    let job_id = 123;
    connect(deps.as_mut(), DAO_CHANNEL_ID);

    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::UpdateIbcTransferRecieverChannel {
            connection_id: "NEW CHANNEL".to_string(),
            channel: Some("newchannel".to_string()),
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    let ack: StdAck = from_binary(&res.acknowledgement).unwrap();

    let channels = query_channels(deps.as_ref(), None, None).unwrap();
    assert_eq!(
        channels,
        IbcTransferChannels {
            endpoints: vec![
                ("connection-two".to_string(), "chan-two".to_string()),
                ("connection-one".to_string(), "chan-one".to_string()),
                (
                    OTHER_CONNECTION_ID.to_string(),
                    OTHER_CHANNEL_ID.to_string(),
                ),
                ("NEW CHANNEL".to_string(), "newchannel".to_string())
            ]
        }
    );

    assert_eq!(0, res.messages.len());
    assert_eq!(
        res.attributes,
        vec![("action", "ibc transfer module updated")]
    );
    assert_eq!(ack, StdAck::Result(to_binary(&job_id).unwrap()));

    // remove
    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::UpdateIbcTransferRecieverChannel {
            connection_id: OTHER_CONNECTION_ID.to_string(),
            channel: None,
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    let channels = query_channels(deps.as_ref(), None, None).unwrap();
    assert_eq!(
        channels,
        IbcTransferChannels {
            endpoints: vec![
                ("connection-two".to_string(), "chan-two".to_string()),
                ("connection-one".to_string(), "chan-one".to_string()),
                ("NEW CHANNEL".to_string(), "newchannel".to_string())
            ]
        }
    );
}
