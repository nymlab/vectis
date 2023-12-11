use cosmwasm_std::{coin, to_binary, BankMsg, CosmosMsg, Empty};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{
    interface::{
        registry_service_trait::sv::RegistryServiceTraitExecMsg,
        wallet_plugin_trait::sv::QueryMsg as WalletPluginQueryMsg,
    },
    types::plugin::PluginListResponse,
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{
                add_test_plugin, create_webauthn_wallet, sign_and_submit,
                sign_and_submit_auth_tx_without_plugins,
            },
        },
    },
};

// This uses the code from contracts/test-contracts/pre-tx-checks
// Pre-tx plugin returns false if msg contains bankmsg,
// `auth_tx_without_plugin` should be able to bypass this check,
// in addition, `auth_tx_without_plugin` should be able to remove this pre_tx_check
#[test]
#[serial]
fn auth_tx_without_plugin_can_bypass_and_remove_plugin() {
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
        suite.test_contracts.pre_tx.2,
    )
    .unwrap();

    // Bypass pre_tx_check by doing a Bank Transfer
    sign_and_submit_auth_tx_without_plugins(
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

    // Can remove plugin
    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: PluginListResponse = wallet.query(&WalletPluginQueryMsg::Plugins {}).unwrap();
    let remove_plugin_msg = RegistryServiceTraitExecMsg::ProxyRemovePlugins {
        addr: info
            .pre_tx
            .iter()
            .map(|(addr, _)| addr.to_string())
            .collect(),
    };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: suite.plugin_registry.to_string(),
            msg: to_binary(&remove_plugin_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();
}
