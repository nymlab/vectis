use cosmwasm_std::{coin, BankMsg, CosmosMsg, Empty};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use test_vectis_post_tx_exec::contract::QueryMsg as PostTxQueryMsg;
use vectis_wallet::{
    interface::{wallet_plugin_trait},
    types::{
        plugin::{PluginListResponse},
    },
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{add_test_plugin, create_webauthn_wallet, sign_and_submit},
        },
    },
};

// This uses the code from contracts/test-contracts/post_tx_exec
// It counts tx that it is plugged into
#[test]
#[serial]
fn post_tx_executes() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    add_test_plugin(
        &app,
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
        suite.test_plugins.post_tx.2,
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let plugins: PluginListResponse = wallet
        .query(&wallet_plugin_trait::QueryMsg::Plugins {})
        .unwrap();

    let post_tx_addr = plugins.post_tx_hooks[0].clone().0;
    let post_tx_contract = Contract::from_addr(&app, post_tx_addr);

    // initially counter is untouched
    let counter: u64 = post_tx_contract
        .query(&PostTxQueryMsg::QueryCounter {})
        .unwrap();
    assert_eq!(counter, 0);

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Bank(BankMsg::Send {
            to_address: VALID_OSMO_ADDR.into(),
            amount: vec![coin(2, DENOM)],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    // post_tx_hook worked
    let counter: u64 = post_tx_contract
        .query(&PostTxQueryMsg::QueryCounter {})
        .unwrap();
    assert_eq!(counter, 1);
}
