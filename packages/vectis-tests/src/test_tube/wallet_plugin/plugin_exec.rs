use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{add_test_plugin, create_webauthn_wallet},
        },
    },
};
use cosmwasm_std::{coin, BankMsg, CosmosMsg, Empty};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Bank, OsmosisTestApp};
use serial_test::serial;
use test_tube::module::Module;
use test_vectis_plugin_exec::contract::ExecMsg;
use vectis_wallet::{interface::wallet_plugin_trait, types::plugin::PluginListResponse};

// This uses the code from contracts/test-contracts/
// It will return false if msg contains bankmsg
#[test]
#[serial]
fn plugin_exec_works() {
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
        suite.test_plugins.exec.2,
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let plugins: PluginListResponse = wallet
        .query(&wallet_plugin_trait::QueryMsg::Plugins {})
        .unwrap();

    let exec_addr = plugins.exec[0].clone().0;
    let exec_contract = Contract::from_addr(&app, exec_addr);

    // check wallet balance before plugin exec
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

    // Plugin contract exec the bank msg on the Proxy
    exec_contract
        .execute(
            &ExecMsg::Exec {
                msgs: vec![CosmosMsg::<Empty>::Bank(BankMsg::Send {
                    to_address: VALID_OSMO_ADDR.into(),
                    amount: vec![coin(2, DENOM)],
                })],
            },
            &[],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    // After balance
    let post_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: wallet.contract_addr.clone(),
            denom: DENOM.into(),
        })
        .unwrap();

    assert_eq!(
        (INIT_BALANCE - 2).to_string(),
        post_balance.balance.unwrap().amount
    );
}
