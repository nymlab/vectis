use crate::common::common::*;
use crate::common::dao_common::*;

#[test]
fn proxy_mint_govec_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    let dao_old_balance = suite.query_balance(&suite.dao).unwrap();
    assert_eq!(dao_old_balance.amount, Uint128::zero());
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // Initially there is something to claim
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory, &wallet_addr)
        .unwrap();
    assert!(unclaimed.is_some());

    // controller mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&WalletFactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![coin(CLAIM_FEE, "ucosm")],
    });

    // Controller execute proxy to claim govec
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute {
                msgs: vec![mint_govec_msg],
            },
            &[coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap();

    let controller_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::from(MINT_AMOUNT));

    // DAO should have received the claim fee and wallet fee
    let dao_current_balance = suite.query_balance(&suite.dao).unwrap();
    assert_eq!(
        dao_old_balance.amount + Uint128::from(CLAIM_FEE) + Uint128::from(WALLET_FEE),
        dao_current_balance.amount
    );

    // Nothing to claim by this wallet
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory, &wallet_addr)
        .unwrap();
    assert_eq!(unclaimed, None);
}

#[test]
fn cannot_mint_govec_without_paying_fee() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // controller mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&WalletFactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![],
    });

    // Initially there is something to claim
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory, &wallet_addr)
        .unwrap();
    assert!(unclaimed.is_some());

    // Controller execute proxy to claim govec
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute {
                msgs: vec![mint_govec_msg],
            },
            &[],
        )
        .unwrap_err();

    let controller_govec_balance = suite.query_govec_balance(&suite.deployer).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::zero());
}

#[test]
fn non_proxy_cannot_mint_on_govec() {
    let mut suite = DaoChainSuite::init().unwrap();

    // controller mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&WalletFactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![coin(CLAIM_FEE, "ucosm")],
    });

    suite
        .app
        .execute_contract(
            suite.deployer.clone(),
            suite.govec.clone(),
            &mint_govec_msg,
            &[coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap_err();

    let controller_govec_balance = suite.query_govec_balance(&suite.deployer).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::zero());
}

#[test]
fn non_proxy_cannot_mint_via_factory() {
    let mut suite = DaoChainSuite::init().unwrap();

    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory, &suite.deployer)
        .unwrap();
    assert!(unclaimed.is_none());

    suite
        .app
        .execute_contract(
            suite.deployer.clone(),
            suite.factory.clone(),
            &WalletFactoryExecuteMsg::ClaimGovec {},
            &[coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap_err();

    let controller_govec_balance = suite.query_govec_balance(&suite.deployer).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::zero());
}

#[test]
fn msg_govec_minted_not_on_dao_chain() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Simulate Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    suite
        .app
        .execute_contract(
            suite.deployer.clone(),
            suite.factory.clone(),
            &WalletFactoryExecuteMsg::GovecMinted {
                success: true,
                wallet_addr: wallet_addr.to_string(),
            },
            &[],
        )
        .unwrap_err();
}

#[test]
fn factory_can_purge_all_expired_claims() {
    let mut suite = DaoChainSuite::init().unwrap();
    let expired = 5;
    let unexpired = 3;
    // Expired after 99 blocks

    for _ in 0..expired {
        suite
            .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
            .unwrap();
    }
    suite.app.update_block(|block| {
        block.time = block
            .time
            .plus_seconds((GOVEC_CLAIM_DURATION_DAY_MUL + 2) * 24 * 60 * 60)
    });

    for _ in 0..unexpired {
        suite
            .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
            .unwrap();
    }

    let all_wallets = suite
        .query_unclaimed_govec_wallets(&suite.factory, None, None)
        .unwrap();
    assert_eq!(all_wallets.wallets.len(), expired + unexpired);

    suite
        .app
        .execute_contract(
            Addr::unchecked("anyone"),
            suite.factory.clone(),
            &WalletFactoryExecuteMsg::PurgeExpiredClaims {
                start_after: Some(all_wallets.wallets[2].0.to_string()),
                limit: None,
            },
            &[],
        )
        .unwrap();

    let all_wallets = suite
        .query_unclaimed_govec_wallets(&suite.factory, None, None)
        .unwrap();
    assert_eq!(all_wallets.wallets.len(), expired + unexpired - 3);

    suite
        .app
        .execute_contract(
            Addr::unchecked("anyone"),
            suite.factory.clone(),
            &WalletFactoryExecuteMsg::PurgeExpiredClaims {
                start_after: None,
                limit: None,
            },
            &[],
        )
        .unwrap();

    let all_wallets = suite
        .query_unclaimed_govec_wallets(&suite.factory, None, None)
        .unwrap();
    assert_eq!(all_wallets.wallets.len(), unexpired);
}
