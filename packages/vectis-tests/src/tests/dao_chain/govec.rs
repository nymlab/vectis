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
    suite
        .proxy_execute(&wallet_addr, vec![transfer_msg], vec![])
        .unwrap_err();

    let other_wallet = Addr::unchecked("other-chain");
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: other_wallet.to_string(),
            },
        )
        .unwrap();

    // ONLY dao-tunnel can relay
    suite
        .govec_execute(
            suite.factory.clone(),
            GovecExecuteMsg::Transfer {
                recipient: suite.dao.to_string(),
                amount: Uint128::one(),
                relayed_from: Some(other_wallet.to_string()),
            },
        )
        .unwrap_err();

    // dao-tunnel can relay
    suite
        .govec_execute(
            suite.dao_tunnel.clone(),
            GovecExecuteMsg::Transfer {
                recipient: suite.dao.to_string(),
                amount: Uint128::one(),
                relayed_from: Some(other_wallet.to_string()),
            },
        )
        .unwrap();

    let wallet_after = suite.query_govec_balance(&other_wallet).unwrap().balance;

    assert_eq!(wallet_after + Uint128::one(), Uint128::from(MINT_AMOUNT),)
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
                msg: send_msg.clone(),
                relayed_from: None,
            },
        )
        .unwrap();
    let after = suite.query_govec_balance(&new_wallet).unwrap().balance;
    assert_eq!(after + Uint128::from(MINT_AMOUNT - 1), before);

    // only dao-tunnel an relay
    suite
        .govec_execute(
            new_wallet.clone(),
            GovecExecuteMsg::Send {
                contract: suite.dao.to_string(),
                amount: Uint128::from(MINT_AMOUNT - 1),
                msg: send_msg.clone(),
                relayed_from: Some(new_wallet.to_string()),
            },
        )
        .unwrap_err();

    // relay works
    suite
        .govec_execute(
            suite.dao_tunnel.clone(),
            GovecExecuteMsg::Send {
                contract: suite.dao.to_string(),
                amount: Uint128::one(),
                msg: send_msg,
                relayed_from: Some(new_wallet.to_string()),
            },
        )
        .unwrap();
    let after_relay = suite.query_govec_balance(&new_wallet).unwrap().balance;
    assert_eq!(after_relay + Uint128::one(), after)
}

#[test]
fn dao_can_update_dao_addr_and_transfer_balance() {
    let mut suite = DaoChainSuite::init().unwrap();
    let new_dao = "new_dao".to_string();

    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: suite.dao.to_string(),
            },
        )
        .unwrap();
    let dao_before = suite.query_govec_balance(&suite.dao).unwrap().balance;

    // Not dao cannot update address
    suite
        .govec_execute(
            suite.deployer.clone(),
            GovecExecuteMsg::UpdateDaoAddr {
                new_addr: new_dao.clone(),
            },
        )
        .unwrap_err();

    // Dao can change address
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::UpdateDaoAddr {
                new_addr: new_dao.clone(),
            },
        )
        .unwrap();

    let dao_after = suite.query_govec_balance(&suite.dao).unwrap().balance;
    let new_dao_after = suite
        .query_govec_balance(&Addr::unchecked(new_dao.clone()))
        .unwrap()
        .balance;

    let r: Addr = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Dao {}).unwrap(),
        }))
        .unwrap();

    assert_eq!(r.to_string(), new_dao);
    assert_eq!(dao_after + Uint128::from(MINT_AMOUNT), dao_before);
    assert_eq!(new_dao_after, Uint128::from(MINT_AMOUNT));
}

#[test]
fn exit_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet = Addr::unchecked("wallet");
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: wallet.to_string(),
            },
        )
        .unwrap();

    suite
        .govec_execute(wallet.clone(), GovecExecuteMsg::Exit { relayed_from: None })
        .unwrap();

    let after_wallet = suite.query_govec_balance(&wallet).unwrap().balance;
    let after_dao = suite.query_govec_balance(&suite.dao).unwrap().balance;

    assert_eq!(after_wallet + Uint128::from(MINT_AMOUNT), after_dao);

    let r: Option<BalanceResponse> = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Joined {
                address: wallet.to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert!(r.is_none());

    // not dao-tunnel cannot relay
    let relayed_wallet = Addr::unchecked("walletr");
    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Mint {
                new_wallet: relayed_wallet.to_string(),
            },
        )
        .unwrap();

    suite
        .govec_execute(
            suite.dao.clone(),
            GovecExecuteMsg::Exit {
                relayed_from: Some(relayed_wallet.to_string()),
            },
        )
        .unwrap_err();

    // dao-tunnel can relay
    suite
        .govec_execute(
            suite.dao_tunnel.clone(),
            GovecExecuteMsg::Exit {
                relayed_from: Some(relayed_wallet.to_string()),
            },
        )
        .unwrap();

    let r: Option<BalanceResponse> = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: suite.govec.to_string(),
            msg: to_binary(&GovecQueryMsg::Joined {
                address: wallet.to_string(),
            })
            .unwrap(),
        }))
        .unwrap();
    assert!(r.is_none());
}
