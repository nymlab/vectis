use assert_matches::assert_matches;
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, Uint128, WasmMsg};
use cw3::Vote;
use cw3_fixed_multisig::msg::ExecuteMsg as MultisigExecuteMsg;
use cw_multi_test::Executor;
use vectis_factory::{msg::ExecuteMsg as FactoryExecuteMsg, ContractError};
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_wallet::{MultiSig, WalletInfo, WalletQueryPrefix};

use crate::common::*;

#[test]
fn create_new_proxy() {
    let init_wallet_fund: Coin = coin(100, "ucosm");

    let mut suite = DaoChainSuite::init().unwrap();

    let init_user_fund = suite
        .query_balance(&Addr::unchecked(USER_ADDR), "ucosm".into())
        .unwrap();
    let init_dao_fund = suite.query_balance(&suite.owner, "ucosm".into()).unwrap();

    let rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![init_wallet_fund.clone()],
        None,
        WALLET_FEE + init_wallet_fund.amount.u128(),
    );
    assert_matches!(rsp, Ok(_));

    let mut r_user = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap();
    let r_all = suite
        .query_all_wallet_addresses(&suite.factory_addr, None, None)
        .unwrap();
    assert_eq!(r_user.wallets[0], r_all.wallets[0]);
    assert_eq!(r_user.wallets.len(), 1);
    assert_eq!(r_all.wallets.len(), 1);

    let wallet_addr = r_user.wallets.pop().unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    let factory_fund = suite
        .query_balance(&suite.factory_addr, "ucosm".into())
        .unwrap();
    let wallet_fund = suite.query_balance(&wallet_addr, "ucosm".into()).unwrap();
    let post_user_fund = suite
        .query_balance(&Addr::unchecked(USER_ADDR), "ucosm".into())
        .unwrap();
    let post_dao_fund = suite.query_balance(&suite.owner, "ucosm".into()).unwrap();

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
    assert_eq!(w.code_id, suite.sc_proxy_id);
    assert!(w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(!w.is_frozen);
}

#[test]
fn cannot_create_new_proxy_without_payment() {
    let no_wallet_fee = 0u128;

    let mut suite = DaoChainSuite::init().unwrap();
    let rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        no_wallet_fee,
    );
    assert!(rsp.is_err());
}

#[test]
fn create_new_proxy_without_guardians() {
    let mut suite = DaoChainSuite::init().unwrap();
    let rsp = suite.create_new_proxy_without_guardians(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );
    assert!(rsp.is_ok());
}

#[test]
fn user_rotate_keys_updates_factory() {
    let mut suite = DaoChainSuite::init().unwrap();

    let rsp = suite.create_new_proxy_without_guardians(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );
    assert!(rsp.is_ok());

    let wallet_address = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap();

    let new_address = "new_key";
    suite
        .app
        .execute_contract(
            Addr::unchecked(USER_ADDR),
            wallet_address.clone(),
            &ProxyExecuteMsg::<Empty>::RotateUserKey {
                new_user_address: new_address.to_string(),
            },
            &[],
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(w.user_addr.as_str(), new_address);

    let origin_user_wallets = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap()
        .wallets
        .pop();

    assert!(origin_user_wallets.is_none());

    let new_user_wallet_addr = suite
        .query_user_wallet_addresses(&suite.factory_addr, new_address, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap();

    assert_eq!(new_user_wallet_addr, wallet_address);
}

#[test]
fn non_wallet_cannot_update_factory() {
    let mut suite = DaoChainSuite::init().unwrap();
    let rsp = suite.create_new_proxy_without_guardians(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );
    assert!(rsp.is_ok());

    let new_address = "new_key";
    let err = suite
        .app
        .execute_contract(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr,
            &FactoryExecuteMsg::UpdateProxyUser {
                new_user: Addr::unchecked(new_address.to_string()),
                old_user: Addr::unchecked(USER_ADDR),
            },
            &[],
        )
        .unwrap_err();

    println!("err {:?}", err);
}
#[test]
fn cannot_create_new_proxy_with_multisig_and_without_guardians_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let multisig = MultiSig {
        threshold_absolute_count: 0,
        multisig_initial_funds: vec![],
    };

    let rsp: ContractError = suite
        .create_new_proxy_without_guardians(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
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
    let mut suite = DaoChainSuite::init().unwrap();
    let init_wallet_fund: Coin = coin(100, "ucosm");
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![init_wallet_fund.clone()],
        None,
        WALLET_FEE + init_wallet_fund.amount.u128(),
    );
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;

    // Can execute Bank msgs
    let send_amount: Coin = coin(10, "ucosm");
    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: suite.factory_addr.to_string(),
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

    let wallet_fund = suite
        .query_balance(&wallet_address, "ucosm".into())
        .unwrap();

    assert_eq!(
        init_wallet_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn create_new_proxy_with_multisig_guardians_can_freeze_wallet() {
    let mut suite = DaoChainSuite::init().unwrap();

    suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
                multisig_initial_funds: vec![],
            }),
            WALLET_FEE,
        )
        .unwrap();

    let mut r = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap();
    let wallet_addr = r.wallets.pop().unwrap();
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
    let mut suite = DaoChainSuite::init().unwrap();
    let init_multisig_fund: Coin = coin(200, "ucosm");
    let init_proxy_fund: Coin = coin(100, "ucosm");
    let init_user_balance = suite.query_balance(&Addr::unchecked(USER_ADDR), "ucosm".to_string());

    suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![init_proxy_fund.clone()],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
                multisig_initial_funds: vec![init_multisig_fund.clone()],
            }),
            WALLET_FEE + init_multisig_fund.amount.u128() + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let mut r = suite
        .query_user_wallet_addresses(&suite.factory_addr, USER_ADDR, None, None)
        .unwrap();
    let proxy_addr = r.wallets.pop().unwrap();

    let w: WalletInfo = suite.query_wallet_info(&proxy_addr).unwrap();
    let multisig_balance = suite.query_balance(&w.multisig_address.unwrap(), "ucosm".to_string());
    let proxy_balance = suite.query_balance(&proxy_addr, "ucosm".to_string());
    let user_balance = suite.query_balance(&Addr::unchecked(USER_ADDR), "ucosm".to_string());
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
fn query_all_wallets() {
    let mut suite = DaoChainSuite::init().unwrap();

    // Create a few wallets
    suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            suite.factory_addr.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    let all = suite
        .query_all_wallet_addresses(&suite.factory_addr, None, None)
        .unwrap();
    let wallet_info: WalletInfo = suite.query_wallet_info(&all.wallets[0]).unwrap();
    let pagination_second = suite
        .query_all_wallet_addresses(
            &suite.factory_addr,
            Some(WalletQueryPrefix {
                user_addr: wallet_info.user_addr.to_string(),
                wallet_addr: all.wallets[0].to_string(),
            }),
            None,
        )
        .unwrap();

    assert_eq!(all.wallets.len(), 3);
    assert_eq!(pagination_second.wallets.len(), 2);
    assert_eq!(all.wallets[1], pagination_second.wallets[0]);
}
