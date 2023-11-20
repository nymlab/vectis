use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{
    interface::{
        registry_management_trait::sv as registry_management_trait,
        wallet_plugin_trait::sv as wallet_plugin_trait,
    },
    types::plugin::{PluginListResponse, PluginWithVersionResponse},
};

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

#[test]
#[serial]
fn can_query_plugin_by_addr() {
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
        suite.test_contracts.post_tx.2,
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let plugins: PluginListResponse = wallet
        .query(&wallet_plugin_trait::QueryMsg::Plugins {})
        .unwrap();

    let post_tx_addr = plugins.post_tx_hooks[0].clone().0;

    let registry = Contract::from_addr(&app, suite.plugin_registry);

    let res: PluginWithVersionResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPluginByAddress {
            contract_addr: post_tx_addr,
        })
        .unwrap();

    assert_eq!(res.contract_version, VECTIS_VERSION);
    assert_eq!(res.plugin_info.id, suite.test_contracts.post_tx.2);
}
