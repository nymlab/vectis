use vectis_contract_tests::common::common::*;
use vectis_contract_tests::common::dao_common::*;
use vectis_govec::state::TokenInfo;
use vectis_wallet::MintResponse;

#[test]
fn transfer_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    let not_a_wallet = Addr::unchecked("not a wallet");
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // controller mint govec
    let mint_govec_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&FactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![coin(CLAIM_FEE, "ucosm")],
    });

    // Controller execute proxy to claim govec
    suite
        .proxy_execute(
            &wallet_addr,
            vec![mint_govec_msg],
            vec![coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap();

    let controller_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::from(MINT_AMOUNT));

    // Controller transfer govec
    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: suite.govec.to_string(),
        msg: to_binary(&GovecExecuteMsg::Transfer {
            recipient: suite.dao.to_string(),
            amount: Uint128::one(),
            relayed_from: None,
        })
        .unwrap(),
        funds: vec![],
    });

    // Controller execute proxy to transfer govec
    suite
        .proxy_execute(&wallet_addr, vec![transfer_msg], vec![])
        .unwrap();

    let balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(balance.balance, Uint128::from(MINT_AMOUNT) - Uint128::one());

    // Controller cannot transfert to no a wallet
    let transfer_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: suite.govec.to_string(),
        msg: to_binary(&GovecExecuteMsg::Transfer {
            recipient: not_a_wallet.to_string(),
            amount: Uint128::one(),
            relayed_from: None,
        })
        .unwrap(),
        funds: vec![],
    });

    // Controller execute proxy to transfer govec
    suite
        .proxy_execute(&wallet_addr, vec![transfer_msg], vec![])
        .unwrap_err();
}

#[test]
fn transfer_from_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // controller mint govec
    let mint_govec_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&FactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![coin(CLAIM_FEE, "ucosm")],
    });

    // Controller execute proxy to claim govec
    suite
        .proxy_execute(
            &wallet_addr,
            vec![mint_govec_msg],
            vec![coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap();

    let controller_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::from(MINT_AMOUNT));

    // Pre-Prop transfer from govec
    suite
        .app
        .execute_contract(
            suite.pre_prop.clone(),
            suite.govec.clone(),
            &GovecExecuteMsg::TransferFrom {
                owner: wallet_addr.to_string(),
                recipient: suite.pre_prop.to_string(),
                amount: Uint128::one(),
            },
            &[],
        )
        .unwrap();

    let balance = suite.query_govec_balance(&wallet_addr).unwrap();
    let balance_pre_prop = suite.query_govec_balance(&suite.pre_prop).unwrap();
    assert_eq!(balance.balance, Uint128::from(MINT_AMOUNT) - Uint128::one());
    assert_eq!(balance_pre_prop.balance, Uint128::one());
}

#[test]
fn query_minter_work() {
    let suite = DaoChainSuite::init().unwrap();

    let r: MintResponse = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Minters {}).unwrap(),
        }))
        .unwrap();
    let minters = r.minters.unwrap();
    assert_eq!(minters.len(), 3);
    assert!(minters.contains(&suite.factory.to_string()));
    assert!(minters.contains(&suite.dao_tunnel.to_string()));
    assert!(minters.contains(&suite.dao.to_string()))
}

#[test]
fn dao_can_update_mint_cap() {
    let mut suite = DaoChainSuite::init().unwrap();
    let r: MintResponse = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Minters {}).unwrap(),
        }))
        .unwrap();
    assert!(r.cap.is_none());

    let new_mint_cap = Some(Uint128::new(123u128));
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::UpdateMintCap {
                new_mint_cap: new_mint_cap.clone(),
            },
        )
        .unwrap();

    let r: MintResponse = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Minters {}).unwrap(),
        }))
        .unwrap();
    assert_eq!(r.cap, new_mint_cap);
}

#[test]
fn govec_can_mint_by_minters_only() {
    let mut suite = DaoChainSuite::init().unwrap();
    let new_wallet = Addr::unchecked("wasm1newwallet");

    let minters = &[
        suite.dao.clone(),
        suite.dao_tunnel.clone(),
        suite.factory.clone(),
    ];
    for minter in minters.iter() {
        let init_balance = suite.query_govec_balance(&new_wallet).unwrap().balance;

        suite
            .govec_execute(
                minter.clone(),
                GovecExecuteMsg::Mint {
                    new_wallet: new_wallet.to_string(),
                },
            )
            .unwrap();

        let after_balance = suite.query_govec_balance(&new_wallet).unwrap().balance;

        assert_eq!(init_balance + Uint128::from(MINT_AMOUNT), after_balance)
    }

    // Not minter should fail
    suite
        .govec_execute(
            suite.deployer.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: new_wallet.to_string(),
            },
        )
        .unwrap_err();

    let info: TokenInfo = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::TokenInfo {}).unwrap(),
        }))
        .unwrap();

    assert_eq!(
        info.total_supply,
        Uint128::from(MINT_AMOUNT) * Uint128::from(minters.len() as u128)
    )
}

#[test]
fn send_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    // create a new wallet and give it some $$
    let new_wallet = Addr::unchecked("wasm1newwallet");
    let not_a_wallet = Addr::unchecked("wasm1notanewwallet");
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: new_wallet.to_string(),
            },
        )
        .unwrap();
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

    // Cannot send to not a wallet
    suite
        .govec_execute(
            new_wallet.clone(),
            GovecExecuteMsg::Send {
                contract: not_a_wallet.to_string(),
                amount: Uint128::from(MINT_AMOUNT),
                msg: send_msg.clone(),
                relayed_from: None,
            },
        )
        .unwrap_err();

    // cannot send more than we have
    suite
        .govec_execute(
            new_wallet.clone(),
            GovecExecuteMsg::Send {
                contract: not_a_wallet.to_string(),
                amount: Uint128::from(MINT_AMOUNT + 12),
                msg: send_msg.clone(),
                relayed_from: None,
            },
        )
        .unwrap_err();

    // checks valid send, dao can receieve even if it did not have a balance
    // before
    let before = suite.query_govec_balance(&new_wallet).unwrap().balance;
    suite
        .govec_execute(
            new_wallet.clone(),
            GovecExecuteMsg::Send {
                contract: suite.dao.to_string(),
                amount: Uint128::from(MINT_AMOUNT - 1),
                msg: send_msg,
                relayed_from: None,
            },
        )
        .unwrap();
    let after = suite.query_govec_balance(&new_wallet).unwrap().balance;
    assert_eq!(after + Uint128::from(MINT_AMOUNT - 1), before)
}
//#[test]
//fn dao_can_update_dao_addr_and_transfer_tokens() {
//    let mut deps = mock_dependencies();
//    let dao_balance = Uint128::new(5);
//    let limit = Uint128::new(12);
//
//    let new_dao = String::from("new_dao");
//
//    do_instantiate(
//        deps.as_mut(),
//        vec![DAO_ADDR],
//        vec![dao_balance],
//        Some(FACTORY),
//        Some(limit),
//        Uint128::new(2),
//        None,
//    );
//    assert_eq!(
//        query_balance(deps.as_ref(), DAO_ADDR.to_string())
//            .unwrap()
//            .balance,
//        Uint128::new(5)
//    );
//    let msg = ExecuteMsg::UpdateConfigAddr {
//        new_addr: UpdateAddrReq::Dao(new_dao.clone()),
//    };
//
//    // only dao can update DAO
//    let info = mock_info(FACTORY, &[]);
//    let env = mock_env();
//    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
//    assert_eq!(err, ContractError::Unauthorized {});
//
//    let info = mock_info(DAO_ADDR, &[]);
//    let env = mock_env();
//    let res = execute(deps.as_mut(), env, info, msg).unwrap();
//    assert_eq!(0, res.messages.len());
//    assert_eq!(query_dao(deps.as_ref()).unwrap(), new_dao);
//    assert_eq!(
//        query_balance(deps.as_ref(), DAO_ADDR.to_string())
//            .unwrap()
//            .balance,
//        Uint128::new(0)
//    );
//    assert_eq!(
//        query_balance(deps.as_ref(), new_dao).unwrap().balance,
//        Uint128::new(5)
//    );
//}
//
//#[test]
//fn transfer() {
//    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
//    let addr1 = String::from("addr0001");
//    let addr2 = String::from("addr0002");
//    let not_wallet = String::from("not_wallet");
//    let amount1 = Uint128::from(12340000u128);
//    let transfer = Uint128::from(76543u128);
//    let too_much = Uint128::from(12340321u128);
//
//    do_instantiate(
//        deps.as_mut(),
//        vec![addr1.as_str(), addr2.as_str()],
//        vec![amount1, Uint128::zero()],
//        Some(FACTORY),
//        None,
//        Uint128::new(2),
//        None,
//    );
//
//    // cannot transfer nothing
//    let info = mock_info(addr1.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: addr2.clone(),
//        amount: Uint128::zero(),
//        relayed_from: None,
//    };
//    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
//    assert_eq!(err, ContractError::InvalidZeroAmount {});
//
//    // cannot send more than we have
//    let info = mock_info(addr1.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: addr2.clone(),
//        amount: too_much,
//        relayed_from: None,
//    };
//    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
//    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));
//
//    // cannot send from empty account
//    let info = mock_info(not_wallet.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: addr1.clone(),
//        amount: transfer,
//        relayed_from: None,
//    };
//    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
//    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));
//
//    // cannot send to non-existing accounts
//    let info = mock_info(addr1.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: not_wallet,
//        amount: transfer,
//        relayed_from: None,
//    };
//    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
//    assert_eq!(err, ContractError::Unauthorized {});
//    assert_eq!(get_balance(deps.as_ref(), addr1.clone()), amount1);
//    assert_eq!(
//        query_token_info(deps.as_ref()).unwrap().total_supply,
//        amount1
//    );
//
//    // valid transfer, aka vote delegation
//    let info = mock_info(addr1.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: addr2.clone(),
//        amount: transfer,
//        relayed_from: None,
//    };
//    let res = execute(deps.as_mut(), env, info, msg).unwrap();
//
//    assert_eq!(
//        res.attributes,
//        [
//            ("action", "transfer"),
//            ("from", &addr1),
//            ("to", &addr2),
//            ("amount", &transfer.to_string())
//        ]
//    );
//
//    assert_eq!(get_balance(deps.as_ref(), addr2), transfer);
//    assert_eq!(
//        query_token_info(deps.as_ref()).unwrap().total_supply,
//        amount1
//    );
//}
//#[test]
//fn remote_relayed_transfer() {
//    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
//    let remote_addr = String::from("remote_addr0001");
//    let dao_addr = String::from("dao_addr0002");
//    let not_wallet = String::from("not_wallet");
//    let amount1 = Uint128::from(12340000u128);
//    let transfer = Uint128::from(76543u128);
//
//    do_instantiate(
//        deps.as_mut(),
//        vec![remote_addr.as_str(), dao_addr.as_str()],
//        vec![amount1, Uint128::zero()],
//        Some(FACTORY),
//        Some(DAO_TUNNEL),
//        None,
//        Uint128::new(2),
//        None,
//    );
//
//    // dao_tunnel can relay transfer
//    let info = mock_info(DAO_TUNNEL, &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Transfer {
//        recipient: dao_addr.clone(),
//        amount: transfer,
//        relayed_from: Some(remote_addr.clone()),
//    };
//    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
//    assert_eq!(
//        res.attributes,
//        [
//            ("action", "transfer"),
//            ("from", &remote_addr),
//            ("to", &dao_addr),
//            ("amount", &transfer.to_string())
//        ]
//    );
//
//    assert_eq!(get_balance(deps.as_ref(), dao_addr.clone()), transfer);
//    assert_eq!(
//        get_balance(deps.as_ref(), remote_addr),
//        amount1.saturating_sub(transfer)
//    );
//    assert_eq!(
//        query_token_info(deps.as_ref()).unwrap().total_supply,
//        amount1
//    );
//
//    // dao_tunnel cannot relay message from non existing wallets
//    let failing_msg = ExecuteMsg::Transfer {
//        recipient: dao_addr.clone(),
//        amount: transfer,
//        relayed_from: Some(not_wallet.clone()),
//    };
//    let err = execute(deps.as_mut(), env.clone(), info, failing_msg).unwrap_err();
//    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));
//
//    // not dao tunnel cannot relay transfers
//    let failing_msg = ExecuteMsg::Transfer {
//        recipient: dao_addr,
//        amount: transfer,
//        relayed_from: Some(not_wallet),
//    };
//    let info_wrong_dao_tunnel = mock_info("not_dao_tunnel", &[]);
//    let err = execute(deps.as_mut(), env, info_wrong_dao_tunnel, failing_msg).unwrap_err();
//    assert_eq!(err, ContractError::Unauthorized {});
//}
//
//#[test]
//fn exit() {
//    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
//
//    let remote_addr = String::from("remote_addr0001");
//
//    do_instantiate(
//        deps.as_mut(),
//        vec![remote_addr.as_str()],
//        vec![Uint128::new(10)],
//        Some(FACTORY),
//        Some(DAO_TUNNEL),
//        None,
//        Uint128::new(2),
//        None,
//    );
//
//    // valid exit update from dao_tunnel remove account from BALANCES
//    let info = mock_info(DAO_TUNNEL, &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Exit {
//        relayed_from: Some(remote_addr.clone()),
//    };
//    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
//    assert_eq!(res.messages.len(), 0);
//
//    let data = query(
//        deps.as_ref(),
//        env.clone(),
//        QueryMsg::Balance {
//            address: remote_addr,
//        },
//    )
//    .unwrap();
//    let balance: BalanceResponse = from_binary(&data).unwrap();
//    assert_eq!(balance.balance, Uint128::new(0));
//
//    let data = query(
//        deps.as_ref(),
//        env.clone(),
//        QueryMsg::Balance {
//            address: DAO_ADDR.to_string(),
//        },
//    )
//    .unwrap();
//    let balance: BalanceResponse = from_binary(&data).unwrap();
//    assert_eq!(balance.balance, Uint128::new(10));
//
//    // invalid dao_tunnel cannot relay
//    let failing_info = mock_info("not_dao_tunnel", &[]);
//    let err = execute(deps.as_mut(), env, failing_info, msg).unwrap_err();
//    assert_eq!(err, ContractError::Unauthorized {});
//}
//
//#[test]
//fn send() {
//    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
//    let addr1 = String::from("addr0001");
//    let addr2 = String::from("addr0002");
//    let amount1 = Uint128::from(12340000u128);
//    let transfer = Uint128::from(76543u128);
//    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());
//
//    do_instantiate(
//        deps.as_mut(),
//        vec![addr1.as_str(), addr2.as_str()],
//        vec![amount1, Uint128::new(0)],
//        Some(FACTORY),
//        Some(DAO_TUNNEL),
//        None,
//        Uint128::new(2),
//        None,
//    );
//
//    // valid transfer to existing addr
//    let info = mock_info(DAO_TUNNEL, &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Send {
//        contract: addr2.to_string(),
//        amount: transfer,
//        msg: send_msg.clone(),
//        relayed_from: Some(addr1.clone()),
//    };
//    let res = execute(deps.as_mut(), env, info, msg).unwrap();
//    assert_eq!(
//        res.attributes,
//        [
//            ("action", "send"),
//            ("from", &addr1),
//            ("to", &addr2),
//            ("amount", &transfer.to_string())
//        ]
//    );
//
//    // ensure proper send message sent
//    // this is the message we want delivered to the other side
//    let binary_msg = Cw20ReceiveMsg {
//        sender: addr1.clone(),
//        amount: transfer,
//        msg: send_msg.clone(),
//    }
//    .into_binary()
//    .unwrap();
//    // and this is how it must be wrapped for the vm to process it
//    assert_eq!(
//        res.messages[0],
//        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//            contract_addr: addr2.to_string(),
//            msg: binary_msg,
//            funds: vec![],
//        }))
//    );
//
//    // ensure balance is properly transferred
//    let remainder = amount1.checked_sub(transfer).unwrap();
//    assert_eq!(get_balance(deps.as_ref(), &addr1), remainder);
//    assert_eq!(get_balance(deps.as_ref(), &addr2), transfer);
//    assert_eq!(
//        query_token_info(deps.as_ref()).unwrap().total_supply,
//        amount1
//    );
//
//    // not dao_tunnel sender cannot relay send
//    let failing_info = mock_info(addr1.as_ref(), &[]);
//    let env = mock_env();
//    let msg = ExecuteMsg::Send {
//        contract: addr2,
//        amount: transfer,
//        msg: send_msg,
//        relayed_from: Some(addr1),
//    };
//    let err = execute(deps.as_mut(), env, failing_info, msg).unwrap_err();
//    assert_eq!(err, ContractError::Unauthorized {});
//}
