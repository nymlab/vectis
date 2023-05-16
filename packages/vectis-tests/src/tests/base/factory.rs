use cosmwasm_std::{coin, Addr, Coin, Uint128};
use vectis_wallet::WalletInfo;

use vectis_contract_tests::common::base_common::*;

#[test]
fn create_new_proxy_works_with_zero_fee() {
    let mut suite = HubChainSuite::init().unwrap();

    // Update wallet fee to 0
    suite
        .update_wallet_fee(suite.deployer.clone(), coin(ZERO_WALLET_FEE, DENOM))
        .unwrap();

    let init_wallet_fund: Coin = coin(100, DENOM);
    let init_controller_fund = suite.query_balance(&suite.controller).unwrap();
    let init_deployer_fund = suite.query_balance(&suite.deployer).unwrap();

    let wallet_addr = suite
        .create_new_proxy_with_default_guardians(
            suite.controller.clone(),
            vec![init_wallet_fund.clone()],
            None,
            coin(ZERO_WALLET_FEE, DENOM),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();
    let wallets_by_controller: Vec<Addr> = suite
        .query_controller_wallet(suite.controller.clone())
        .unwrap();

    let factory_fund = suite.query_balance(&suite.factory).unwrap();
    let wallet_fund = suite.query_balance(&wallet_addr).unwrap();
    let post_controller_fund = suite.query_balance(&suite.controller.clone()).unwrap();
    let post_deployer_fund = suite.query_balance(&suite.deployer).unwrap();

    assert!(wallets_by_controller.contains(&wallet_addr));
    // factory fund does not change
    assert_eq!(Uint128::zero(), factory_fund.amount,);
    // wallet fund should be what is specified
    assert_eq!(wallet_fund.amount, init_wallet_fund.amount,);
    // controller funds should be wallet_fee + init wallet fund less
    assert_eq!(
        init_controller_fund.amount.u128() - post_controller_fund.amount.u128(),
        init_wallet_fund.amount.u128()
    );
    // deployer fund should increase by wallet_fee
    assert_eq!(
        post_deployer_fund.amount.u128() - init_deployer_fund.amount.u128(),
        ZERO_WALLET_FEE
    );
    // initial states should match creation params
    assert!(w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(!w.is_frozen);
}

#[test]
fn cannot_send_too_much_fee() {
    let mut suite = HubChainSuite::init().unwrap();

    // Update wallet fee to 0
    suite
        .update_wallet_fee(suite.deployer.clone(), coin(ZERO_WALLET_FEE, DENOM))
        .unwrap();

    let init_wallet_fund: Coin = coin(100, DENOM);

    suite
        .create_new_proxy_with_default_guardians(
            suite.controller.clone(),
            vec![init_wallet_fund.clone()],
            None,
            coin(WALLET_FEE, DENOM),
        )
        .unwrap_err();
}
