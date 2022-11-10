use crate::common::common::*;
use crate::common::dao_common::*;

#[test]
fn user_mint_govec_works() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(
            suite.user.clone(),
            suite.factory.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    // user mint govec
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

    // User execute proxy to claim govec
    let res = suite.app.execute_contract(
        suite.user.clone(),
        wallet_addr.clone(),
        &ProxyExecuteMsg::Execute {
            msgs: vec![mint_govec_msg],
        },
        &[],
    );

    // TODO check events

    let user_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(user_govec_balance.balance, Uint128::from(MINT_AMOUNT));

    // Nothing to claim by this wallet
    let unclaimed = suite
        .query_proxy_govec_claim_expiration(&suite.factory, &wallet_addr)
        .unwrap();
    assert_eq!(unclaimed, None);
}
