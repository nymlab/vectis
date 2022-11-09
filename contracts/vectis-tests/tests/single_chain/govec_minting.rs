use crate::common::common::*;
use crate::common::dao_common::*;

#[test]
#[cfg(feature = "dao-chain")]
fn user_mint_govec_works() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    // user mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory_addr.to_string(),
        msg: to_binary(&WalletFactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![],
    });

    // Initially there is something to claim
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory_addr, &wallet_addr)
        .unwrap();
    assert!(unclaimed.is_some());

    // User execute proxy to claim govec
    let res = suite.app.execute_contract(
        Addr::unchecked(USER_ADDR),
        wallet_addr.clone(),
        &ProxyExecuteMsg::Execute {
            msgs: vec![mint_govec_msg],
        },
        &[],
    );

    // TODO check events

    let user_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(user_govec_balance.balance, Uint128::one());

    // Nothing to claim by this wallet
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory_addr, &wallet_addr)
        .unwrap();
    assert_eq!(unclaimed, None);
}

#[test]
#[cfg(feature = "remote")]
fn remote_handles_successful_govec_minting() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    // user mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory_addr.to_string(),
        msg: to_binary(&WalletFactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![],
    });

    // User execute proxy to claim govec
    let res = suite
        .app
        .execute_contract(
            Addr::unchecked(USER_ADDR),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute {
                msgs: vec![mint_govec_msg],
            },
            &[],
        )
        .unwrap();

    // factory sends message to remote tunnel
    assert!(res.has_event(
        &Event::new("wasm")
            .add_attribute("action", "mint_govec requested")
            .add_attribute("wallet_addr", wallet_addr.to_string())
    ));

    // Mock dao tunnel recieves remote tunnel msg

    // Mock remote tunnel receives dao tunnel ack

    // factory handles remote tunnel message

    //     let user_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    //     assert_eq!(user_govec_balance.balance, Uint128::one());
    //
    //     // Nothing to claim by this wallet
    //     let unclaimed = suite
    //         .query_proxy_govec_claim_expiration(&suite.factory_addr, &wallet_addr)
    //         .unwrap();
    //     assert_eq!(unclaimed, None);
}
