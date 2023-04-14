use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Coin, CosmosMsg, Empty, Uint128, WasmMsg};
use cw3::Vote;
use cw3_fixed_multisig::msg::ExecuteMsg as MultisigExecuteMsg;
use vectis_factory::ContractError;
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_wallet::{MultiSig, WalletInfo};

use vectis_contract_tests::common::base_common::*;

#[test]
fn create_new_proxy() {
    let init_wallet_fund: Coin = coin(100, DENOM);

    let mut suite = HubChainSuite::init().unwrap();

    let init_controller_fund = suite.query_balance(&suite.controller).unwrap();
    let init_deployer_fund = suite.query_balance(&suite.deployer).unwrap();

    let wallet_addr = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_wallet_fund.clone()],
            None,
            WALLET_FEE + init_wallet_fund.amount.u128(),
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
        WALLET_FEE + init_wallet_fund.amount.u128()
    );
    // deployer fund should increase by wallet_fee
    assert_eq!(
        post_deployer_fund.amount.u128() - init_deployer_fund.amount.u128(),
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

    let mut suite = HubChainSuite::init().unwrap();
    suite
        .create_new_proxy(suite.controller.clone(), vec![], None, no_wallet_fee)
        .unwrap();
}

#[test]
fn create_new_proxy_without_guardians() {
    let mut suite = HubChainSuite::init().unwrap();
    suite
        .create_new_proxy_without_guardians(
            suite.controller.clone(),
            suite.factory.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();
}

#[test]
fn controller_can_rotate_keys() {
    let mut suite = HubChainSuite::init().unwrap();

    let wallet_address = suite
        .create_new_proxy_without_guardians(
            suite.controller.clone(),
            suite.factory.clone(),
            vec![],
            None,
            WALLET_FEE,
        )
        .unwrap();

    let new_address = "new_key";
    let old_controller_wallets = suite
        .query_controller_wallet(suite.controller.clone())
        .unwrap();
    let new_controller_wallets = suite
        .query_controller_wallet(Addr::unchecked(new_address))
        .unwrap();
    assert!(old_controller_wallets.contains(&wallet_address));
    assert!(!new_controller_wallets.contains(&wallet_address));

    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_address.clone(),
            &ProxyExecuteMsg::<Empty>::RotateControllerKey {
                new_controller_address: new_address.to_string(),
            },
            &[],
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(w.controller_addr.as_str(), new_address);
    let old_controller_wallets = suite
        .query_controller_wallet(suite.controller.clone())
        .unwrap();
    let new_controller_wallets = suite
        .query_controller_wallet(Addr::unchecked(new_address))
        .unwrap();

    assert!(!old_controller_wallets.contains(&wallet_address));
    assert!(new_controller_wallets.contains(&wallet_address))
}

#[test]
fn cannot_create_new_proxy_with_multisig_and_without_guardians_fails() {
    let mut suite = HubChainSuite::init().unwrap();
    let multisig = MultiSig {
        threshold_absolute_count: 0,
    };

    let rsp: ContractError = suite
        .create_new_proxy_without_guardians(
            suite.controller.clone(),
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
fn controller_can_execute_messages() {
    let mut suite = HubChainSuite::init().unwrap();
    let init_wallet_fund: Coin = coin(100, DENOM);
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_wallet_fund.clone()],
            None,
            WALLET_FEE + init_wallet_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let controller = w.controller_addr;

    // Can execute Bank msgs
    let send_amount: Coin = coin(10, DENOM);
    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: suite.factory.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        controller,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute { msgs: vec![msg] },
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
    let mut suite = HubChainSuite::init().unwrap();

    let wallet_addr = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
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
