use cosmwasm_std::testing::mock_ibc_packet_timeout;

use crate::{
    contract::{query_channels, query_dao_config},
    ibc::ibc_packet_timeout,
    msg::{ChainConfigResponse, ExecuteMsg, Receiver},
    tests::*,
};

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
                channel,
                counterparty_version,
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
                channel,
                counterparty_version,
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

    // connected in the previous step
    let dao_config = query_dao_config(deps.as_ref()).unwrap();
    assert!(dao_config.dao_tunnel_channel.is_some());

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
    );

    // removed channel_id
    let dao_config = query_dao_config(deps.as_ref()).unwrap();
    assert_eq!(dao_config.dao_tunnel_channel, None);
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
        claim_fee: Coin {
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

    let res: ChainConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::ChainConfig {}).unwrap()).unwrap();

    assert_eq!(contract_address.to_string(), res.remote_factory.unwrap())
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

#[test]
fn handle_dispatch_actions_ack_with_job_id() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let job_id = 222;

    // This sets states so that ibc packet can be received
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let coin = Coin {
        amount: Uint128::from(11u32),
        denom: DENOM.to_string(),
    };

    let dispatch_bank = CosmosMsg::Bank(BankMsg::Send {
        to_address: "to_address".to_string(),
        amount: vec![coin.clone()],
    });

    let dispatch_self_ibc_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::IbcTransfer {
            receiver: Receiver {
                connection_id: OTHER_CONNECTION_ID.to_string(),
                addr: DAO_ADDR.to_string(),
            },
        })
        .unwrap(),
        funds: vec![coin],
    });
    let dispatch_msgs = vec![dispatch_bank, dispatch_self_ibc_transfer];

    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::DispatchActions {
            msgs: dispatch_msgs,
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
    let ack: StdAck = from_binary(&res.acknowledgement).unwrap();

    assert_eq!(DISPATCH_CALLBACK_ID, res.messages[0].id.clone());
    assert_eq!(DISPATCH_CALLBACK_ID, res.messages[1].id.clone());
    assert_eq!(ack, StdAck::Result(to_binary(&job_id).unwrap()));
}

#[test]
fn handle_dispatch_actions_ack_failure() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let job_id = 222;
    connect(deps.as_mut(), DAO_CHANNEL_ID);

    // this should fail as no fund is given
    let dispatch_self_ibc_transfer = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&ExecuteMsg::IbcTransfer {
            receiver: Receiver {
                connection_id: "notset".to_string(),
                addr: DAO_ADDR.to_string(),
            },
        })
        .unwrap(),
        funds: vec![],
    });

    let ibc_msg = PacketMsg {
        sender: DAO_ADDR.to_string(),
        job_id,
        msg: to_binary(&DaoTunnelPacketMsg::DispatchActions {
            msgs: vec![dispatch_self_ibc_transfer],
        })
        .unwrap(),
    };

    let msg = mock_ibc_packet_recv(DAO_CHANNEL_ID, &ibc_msg).unwrap();
    let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

    let response = Reply {
        id: res.messages[0].id,
        result: SubMsgResult::Err("reply err".to_string()),
    };

    let res = reply(deps.as_mut(), mock_env(), response).unwrap();

    let ack: StdAck = from_binary(&res.data.unwrap()).unwrap();

    // reply only on error for DISPATCH_CALLBACK_ID
    assert_eq!(ack, StdAck::Error("reply err".to_string()));
}

// Test `ibc_packet_ack`
#[test]
fn mint_govec_packet_ack_sends_msg_to_factory() {
    let mut deps = do_instantiate();
    let job_id = 2901;
    let action_id = 10;
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let env = mock_env();

    // in the case of success
    let wallet_msg = WalletFactoryExecuteMsg::GovecMinted {
        success: true,
        wallet_addr: "wallet".to_string(),
    };
    let ack = StdAck::success(action_id);
    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: "wallet".to_string(),
        })
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_ack(
        DAO_CHANNEL_ID,
        &original_ibc_msg,
        IbcAcknowledgement::new(ack),
    )
    .unwrap();
    let res = ibc_packet_ack(deps.as_mut(), env.clone(), ibc_ack).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("action", "Mint Govec Ack".to_string()),
            ("success", true.to_string())
        ]
    );

    let sub_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&wallet_msg).unwrap(),
        funds: vec![],
    };
    assert_eq!(res.messages[0], SubMsg::new(sub_msg));

    // in the case of failure
    let ack = StdAck::fail("some error".to_string());
    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: "wallet".to_string(),
        })
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_ack(
        DAO_CHANNEL_ID,
        &original_ibc_msg,
        IbcAcknowledgement::new(ack),
    )
    .unwrap();
    let res = ibc_packet_ack(deps.as_mut(), env.clone(), ibc_ack).unwrap();

    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("action", "Mint Govec Ack".to_string()),
            ("success", false.to_string())
        ]
    );

    let wallet_msg = WalletFactoryExecuteMsg::GovecMinted {
        success: false,
        wallet_addr: "wallet".to_string(),
    };
    let sub_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&wallet_msg).unwrap(),
        funds: vec![],
    };
    assert_eq!(res.messages[0], SubMsg::new(sub_msg))
}

#[test]
fn handle_ack_for_non_govec_minting_packets() {
    let mut deps = do_instantiate();
    let job_id = 2901;
    let action_id = VectisDaoActionIds::GovecBurn;
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let env = mock_env();

    // In the case of success
    let ack = StdAck::success(action_id.clone() as u64);
    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::GovecActions(
            vectis_wallet::GovecExecuteMsg::Burn {
                relayed_from: Some("proxy".to_string()),
            },
        ))
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_ack(
        DAO_CHANNEL_ID,
        &original_ibc_msg,
        IbcAcknowledgement::new(ack),
    )
    .unwrap();
    let res = ibc_packet_ack(deps.as_mut(), env.clone(), ibc_ack).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("result", format!("{:?}", action_id))
        ]
    );

    // In the case of failure
    let ack = StdAck::fail("failed msg".to_string());
    let ibc_ack = mock_ibc_packet_ack(
        DAO_CHANNEL_ID,
        &original_ibc_msg,
        IbcAcknowledgement::new(ack),
    )
    .unwrap();
    let res = ibc_packet_ack(deps.as_mut(), env, ibc_ack).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("result", "failed msg".to_string())
        ]
    )
}

#[test]
fn handle_timeout_for_govec_minting() {
    let mut deps = do_instantiate();
    let job_id = 2901;
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let env = mock_env();

    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::MintGovec {
            wallet_addr: "wallet".to_string(),
        })
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_timeout(DAO_CHANNEL_ID, &original_ibc_msg).unwrap();
    let res = ibc_packet_timeout(deps.as_mut(), env.clone(), ibc_ack).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("action", "Mint Govec Timeout: revert".to_string())
        ]
    );

    let wallet_msg = WalletFactoryExecuteMsg::GovecMinted {
        success: false,
        wallet_addr: "wallet".to_string(),
    };
    let sub_msg = WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        msg: to_binary(&wallet_msg).unwrap(),
        funds: vec![],
    };
    assert_eq!(res.messages[0], SubMsg::new(sub_msg));
}

#[test]
fn handle_timeout_for_non_govec_minting_packets() {
    let mut deps = do_instantiate();
    let job_id = 2901;
    connect(deps.as_mut(), DAO_CHANNEL_ID);
    let env = mock_env();

    let original_ibc_msg = PacketMsg {
        sender: env.contract.address.to_string(),
        job_id,
        msg: to_binary(&RemoteTunnelPacketMsg::GovecActions(
            vectis_wallet::GovecExecuteMsg::Burn {
                relayed_from: Some("proxy".to_string()),
            },
        ))
        .unwrap(),
    };

    let ibc_ack = mock_ibc_packet_timeout(DAO_CHANNEL_ID, &original_ibc_msg).unwrap();
    let res = ibc_packet_timeout(deps.as_mut(), env, ibc_ack).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            ("job_id", job_id.to_string()),
            ("action", "Ibc Timeout".to_string())
        ]
    )
}
