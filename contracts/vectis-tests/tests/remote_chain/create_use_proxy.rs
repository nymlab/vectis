use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, Uint128, WasmMsg};
use cw3::Vote;
use cw3_fixed_multisig::msg::ExecuteMsg as MultisigExecuteMsg;
use cw_multi_test::Executor;
use cw_utils::Expiration;
use vectis_factory::ContractError;
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_wallet::{MultiSig, WalletInfo};

use crate::common::common::*;
use crate::common::remote_common::*;

#[test]
fn create_new_proxy() {
    let init_wallet_fund: Coin = coin(100, "uremote");

    let mut suite = RemoteChainSuite::init().unwrap();

    let init_user_fund = suite.query_balance(&suite.user).unwrap();
    let init_dao_fund = suite.query_balance(&suite.remote_tunnel).unwrap();

    let wallet_addr = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_wallet_fund.clone()],
            None,
            WALLET_FEE + init_wallet_fund.amount.u128(),
        )
        .unwrap();

    // Checks unclaimed_wallet_list has added new proxy
    let unclaimed_wallet_list = suite
        .query_unclaimed_govec_wallets(&suite.factory, None, None)
        .unwrap();
    assert_eq!(unclaimed_wallet_list.wallets.len(), 1);
    // Checks expiration is created correctly
    if let Expiration::AtTime(time) = unclaimed_wallet_list.wallets[0].1 {
        let expiration = suite.claim_expiration();
        assert_eq!(time, suite.app.block_info().time.plus_seconds(expiration));
    } else {
        assert!(false);
    }

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    let factory_fund = suite.query_balance(&suite.factory).unwrap();
    let wallet_fund = suite.query_balance(&wallet_addr).unwrap();
    let post_user_fund = suite.query_balance(&suite.user.clone()).unwrap();
    let post_dao_fund = suite.query_balance(&suite.remote_tunnel).unwrap();

    // factory fund does not change
    assert_eq!(Uint128::zero(), factory_fund.amount,);
    // wallet fund should be what is specified
    assert_eq!(wallet_fund.amount, init_wallet_fund.amount,);
    // user funds should be wallet_fee + init wallet fund less
    assert_eq!(
        init_user_fund.amount.u128() - post_user_fund.amount.u128(),
        WALLET_FEE + init_wallet_fund.amount.u128()
    );
    // dao fund should increase by wallet_fee
    assert_eq!(
        post_dao_fund.amount.u128() - init_dao_fund.amount.u128(),
        WALLET_FEE
    );
    // initial states should match creation params
    assert!(w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(!w.is_frozen);
}

#[test]
#[should_panic]
fn cannot_create_new_proxy_without_payment() {
    let no_wallet_fee = 0u128;

    let mut suite = RemoteChainSuite::init().unwrap();
    suite
        .create_new_proxy(suite.user.clone(), vec![], None, no_wallet_fee)
        .unwrap();
}

#[test]
fn create_new_proxy_without_guardians() {
    let mut suite = RemoteChainSuite::init().unwrap();
    suite
        .create_new_proxy_without_guardians(
            suite.user.clone(),
            suite.factory.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();
}

#[test]
fn user_can_rotate_keys() {
    let mut suite = RemoteChainSuite::init().unwrap();

    let wallet_address = suite
        .create_new_proxy_without_guardians(
            suite.user.clone(),
            suite.factory.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    let new_address = "new_key";
    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_address.clone(),
            &ProxyExecuteMsg::<Empty>::RotateUserKey {
                new_user_address: new_address.to_string(),
            },
            &[],
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(w.user_addr.as_str(), new_address);
}

#[test]
fn cannot_create_new_proxy_with_multisig_and_without_guardians_fails() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let multisig = MultiSig {
        threshold_absolute_count: 0,
        multisig_initial_funds: vec![],
    };

    let rsp: ContractError = suite
        .create_new_proxy_without_guardians(
            suite.user.clone(),
            suite.factory.clone(),
            vec![],
            Some(multisig),
            10,
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        rsp.to_string(),
        String::from("ThresholdShouldBeGreaterThenZero")
    );
}

#[test]
fn user_can_execute_messages() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let init_wallet_fund: Coin = coin(100, "uremote");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_wallet_fund.clone()],
            None,
            WALLET_FEE + init_wallet_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;

    // Can execute Bank msgs
    let send_amount: Coin = coin(10, "uremote");
    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: suite.factory.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        user,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute {
            msgs: vec![msg.clone()],
        },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let wallet_fund = suite.query_balance(&wallet_address).unwrap();

    assert_eq!(
        init_wallet_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn create_new_proxy_with_multisig_guardians_can_freeze_wallet() {
    let mut suite = RemoteChainSuite::init().unwrap();

    let wallet_addr = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
                multisig_initial_funds: vec![],
            }),
            WALLET_FEE,
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();
    assert!(!w.is_frozen);

    // Create proposal and vote for freezing
    let multisig_contract_addr = w.multisig_address.unwrap();
    let execute_revert_freeze_status_msg = WasmMsg::Execute {
        contract_addr: wallet_addr.to_string(),
        msg: to_binary(&ProxyExecuteMsg::<Empty>::RevertFreezeStatus {}).unwrap(),
        funds: vec![],
    };

    let multisig_propose_msg = MultisigExecuteMsg::Propose {
        title: "Revert freeze status".to_string(),
        description: "Need to revert freeze status".to_string(),
        msgs: vec![execute_revert_freeze_status_msg.into()],
        latest: None,
    };

    // propose wallet revert freeze status
    // first proposer has considered cast a ballot
    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD1),
            multisig_contract_addr.clone(),
            &multisig_propose_msg,
            &[],
        )
        .unwrap();

    // vote msg
    let vote_msg = MultisigExecuteMsg::Vote {
        proposal_id: 1,
        vote: Vote::Yes,
    };

    // second vote
    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD2),
            multisig_contract_addr.clone(),
            &vote_msg,
            &[],
        )
        .unwrap();

    // execute proposal
    let execute_proposal_msg = MultisigExecuteMsg::Execute { proposal_id: 1 };

    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD1),
            multisig_contract_addr,
            &execute_proposal_msg,
            &[],
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    // Ensure freezing msg passed
    assert!(w.is_frozen);
}

#[test]
fn create_new_proxy_with_multisig_guardians_has_correct_fund() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let init_multisig_fund: Coin = coin(200, "uremote");
    let init_proxy_fund: Coin = coin(100, "uremote");
    let init_user_balance = suite.query_balance(&suite.user.clone());

    let proxy_addr = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
                multisig_initial_funds: vec![init_multisig_fund.clone()],
            }),
            WALLET_FEE + init_multisig_fund.amount.u128() + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&proxy_addr).unwrap();
    let multisig_balance = suite.query_balance(&w.multisig_address.unwrap());
    let proxy_balance = suite.query_balance(&proxy_addr);
    let user_balance = suite.query_balance(&suite.user.clone());
    assert_eq!(multisig_balance.unwrap().amount, init_multisig_fund.amount);
    assert_eq!(proxy_balance.unwrap().amount, init_proxy_fund.amount);
    assert_eq!(
        user_balance.unwrap().amount.u128()
            + WALLET_FEE
            + init_proxy_fund.amount.u128()
            + init_multisig_fund.amount.u128(),
        init_user_balance.unwrap().amount.u128()
    );
}

#[test]
fn query_all_unclaimed_wallets_works() {
    let mut suite = RemoteChainSuite::init().unwrap();

    // Create a few wallets
    suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    let all = suite
        .query_unclaimed_govec_wallets(&suite.factory, None, None)
        .unwrap();
    let pagination_second = suite
        .query_unclaimed_govec_wallets(&suite.factory, Some(all.wallets[0].0.to_string()), None)
        .unwrap();

    assert_eq!(all.wallets.len(), 3);
    assert_eq!(pagination_second.wallets.len(), 2);
    assert_eq!(all.wallets[1], pagination_second.wallets[0]);
}
