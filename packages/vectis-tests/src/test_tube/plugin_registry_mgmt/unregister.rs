use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, vectis_committee},
    },
};
use cosmwasm_std::coin;
use osmosis_test_tube::OsmosisTestApp;
use vectis_wallet::{interface::registry_management_trait, types::plugin::PluginsResponse};

#[test]
fn can_unregister_with_correct_params() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let code_data =
        test_plugin_code_data(suite.test_contracts.pre_tx.0, suite.test_contracts.pre_tx.1);

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data,
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();

    assert_eq!(plugins.total, 1);
    assert_eq!(plugins.current_plugin_id, 1);

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::UnregisterPlugin { id: 1 },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();

    assert_eq!(plugins.total, 0);
    assert_eq!(plugins.current_plugin_id, 1);
}

#[test]
fn cannot_unregister_not_existing() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();
    assert_eq!(plugins.total, 0);

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry,
        &registry_management_trait::ExecMsg::UnregisterPlugin { id: 1 },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap_err();
}

#[test]
fn not_deployer_cannot_unregister() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let code_data =
        test_plugin_code_data(suite.test_contracts.pre_tx.0, suite.test_contracts.pre_tx.1);

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data: code_data.clone(),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry);
    // Sent from another signer
    registry
        .execute(
            &registry_management_trait::ExecMsg::RegisterPlugin {
                code_data,
                metadata_data: test_plugin_metadata(),
            },
            &[coin(REGISTRY_FEE, "uosmo")],
            &suite.accounts[ICOMMITTEE],
        )
        .unwrap_err();
}
