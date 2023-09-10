use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, vectis_committee},
    },
};
use cosmwasm_std::coin;
use osmosis_test_tube::OsmosisTestApp;
use vectis_wallet::{
    interface::registry_management_trait,
    types::plugin::{Plugin, PluginMetadataData, PluginsResponse},
};

#[test]
fn can_update_metadata_with_correct_params() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let new_display_name = "vectis-new-display-name-test";
    let new_ipfs_hash = "vectis-new-ipfs-hash-test";

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data: test_plugin_code_data(suite.test_plugin_code_id),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let plugin: Plugin = registry
        .query(&registry_management_trait::QueryMsg::GetPluginById { id: 1 })
        .unwrap();

    assert_ne!(plugin.display_name, new_display_name);
    assert_ne!(
        plugin.get_latest_version_details().unwrap().ipfs_hash,
        new_ipfs_hash
    );

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::NewPluginVersion {
            id: 1,
            code_update: None,
            metadata_update: PluginMetadataData {
                creator: VALID_OSMO_ADDR.into(),
                display_name: new_display_name.into(),
                ipfs_hash: new_ipfs_hash.into(),
            },
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let plugin: Plugin = registry
        .query(&registry_management_trait::QueryMsg::GetPluginById { id: 1 })
        .unwrap();

    assert_eq!(plugin.display_name, new_display_name);
    assert_eq!(
        plugin.get_latest_version_details().unwrap().ipfs_hash,
        new_ipfs_hash
    );
}

#[test]
fn cannot_update_not_existing() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let new_display_name = "vectis-new-display-name-test";
    let new_ipfs_hash = "vectis-new-ipfs-hash-test";

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
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::NewPluginVersion {
            id: 1,
            code_update: None,
            metadata_update: PluginMetadataData {
                creator: VALID_OSMO_ADDR.into(),
                display_name: new_display_name.into(),
                ipfs_hash: new_ipfs_hash.into(),
            },
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap_err();
}

#[test]
fn not_deployer_cannot_update() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data: test_plugin_code_data(suite.test_plugin_code_id),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    registry
        .execute(
            &registry_management_trait::ExecMsg::RegisterPlugin {
                code_data: test_plugin_code_data(suite.test_plugin_code_id),
                metadata_data: test_plugin_metadata(),
            },
            &[coin(REGISTRY_FEE, "uosmo")],
            &suite.accounts[ICOMMITTEE],
        )
        .unwrap_err();
}

#[test]
fn todo_cannot_update_incorrect_code_data() {}

#[test]
fn todo_can_update_correct_code_data() {}
