use cosmwasm_std::{coin, BankMsg, CosmosMsg, Empty};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Account, Bank, OsmosisTestApp};
use serial_test::serial;
use test_tube::module::Module;

use vectis_wallet::{
    interface::wallet_trait::sv::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    types::wallet::{Nonce, WalletInfo},
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            msgs::simple_bank_send,
            wallet::{create_webauthn_wallet, sign_and_create_relay_tx},
        },
    },
};

#[test]
#[serial]
fn wallet_can_do_webauthn_tx_success() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid = "test-user";
    let transfer = coin(5, DENOM);

    let (wallet_addr, pubkey) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );
    let wallet = Contract::from_addr(&app, wallet_addr.into_string());

    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();
    assert_eq!(info.controller.nonce, 0);
    assert_eq!(info.controller.data, pubkey.as_slice());

    let bank = Bank::new(&app);
    let init_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: wallet.contract_addr.clone(),
            denom: DENOM.into(),
        })
        .unwrap();
    assert_eq!(
        init_balance.balance.unwrap().amount,
        INIT_BALANCE.to_string()
    );

    // ===========================
    // Signing and create tx data
    // ===========================
    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Bank(BankMsg::Send {
            to_address: suite.accounts[IDEPLOYER].address(),
            amount: vec![transfer.clone()],
        })],
        info.controller.nonce,
        vid,
    );

    wallet
        .execute(
            &WalletExecMsg::AuthExec {
                transaction: relay_tx,
            },
            &[],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    let post_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: wallet.contract_addr.clone(),
            denom: DENOM.into(),
        })
        .unwrap();

    assert_eq!(
        (INIT_BALANCE - transfer.amount.u128()).to_string(),
        post_balance.balance.unwrap().amount
    )
}

#[test]
#[serial]
fn wrong_nonce_signature_fails() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid = "test-user";
    let test_nonce: Nonce = 99;

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );
    let wallet = Contract::from_addr(&app, wallet_addr.into_string());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();
    assert_ne!(info.controller.nonce, test_nonce);

    // ===========================
    // Signing and create tx data
    // ===========================
    let relay_tx = sign_and_create_relay_tx(vec![simple_bank_send()], test_nonce, vid);

    let res = wallet.execute(
        &WalletExecMsg::AuthExec {
            transaction: relay_tx,
        },
        &[],
        &suite.accounts[IRELAYER],
    );
    assert!(res.is_err())
}

#[test]
#[serial]
fn wrong_entity_signature_fails() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid_1 = "test-user-1";
    let vid_2 = "test-user-2";

    let (wallet_addr_1, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid_1,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );
    let _ = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid_2,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let wallet_1 = Contract::from_addr(&app, wallet_addr_1.into_string());

    // sign by wrong user
    let relay_tx = sign_and_create_relay_tx(vec![simple_bank_send()], 0, vid_2);

    let res = wallet_1.execute(
        &WalletExecMsg::AuthExec {
            transaction: relay_tx,
        },
        &[],
        &suite.accounts[IRELAYER],
    );
    assert!(res.is_err())
}
