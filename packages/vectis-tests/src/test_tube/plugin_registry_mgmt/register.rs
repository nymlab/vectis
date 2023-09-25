use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, vectis_committee},
    },
};
use cosmwasm_std::coin;
use osmosis_test_tube::OsmosisTestApp;
use std::collections::BTreeMap;
use vectis_wallet::{
    interface::registry_management_trait,
    types::plugin::{Plugin, PluginsResponse, VersionDetails},
};

#[test]
fn can_register_with_correct_params() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data: test_plugin_code_data(
                suite.test_contracts.pre_tx.0,
                suite.test_contracts.pre_tx.1,
            ),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry);

    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();

    assert_eq!(plugins.total, 1);
    assert_eq!(plugins.current_plugin_id, 1);

    let plugin: Plugin = registry
        .query(&registry_management_trait::QueryMsg::GetPluginById { id: 1 })
        .unwrap();

    let code_data =
        test_plugin_code_data(suite.test_contracts.pre_tx.0, suite.test_contracts.pre_tx.1);
    let expected_plugin = Plugin {
        id: 1,
        creator: canonical_valid_osmo(),
        display_name: test_plugin_metadata().display_name,
        latest_contract_version: code_data.latest_contract_version.clone(),
        versions: BTreeMap::from([(
            code_data.latest_contract_version,
            VersionDetails {
                code_id: code_data.new_code_id,
                code_hash: code_data.new_code_hash,
                ipfs_hash: test_plugin_metadata().ipfs_hash,
            },
        )]),
    };

    assert_eq!(plugins.plugins[0], expected_plugin);
    assert_eq!(plugin, expected_plugin)
}

#[test]
fn cannot_register_without_correct_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let code_data =
        test_plugin_code_data(suite.test_contracts.pre_tx.0, suite.test_contracts.pre_tx.1);

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data,
            metadata_data: test_plugin_metadata(),
        },
        // INCORRECT fee
        &[coin(WALLET_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap_err();
}

#[test]
fn cannot_register_incorrect_code_hash() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    // WRONG codeID - mismatch to codehash
    let code_data = test_plugin_code_data(
        suite.test_contracts.post_tx.0,
        suite.test_contracts.pre_tx.1,
    );

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data,
            metadata_data: test_plugin_metadata(),
        },
        // INCORRECT fee
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap_err();
}

#[test]
fn not_deployer_cannot_register() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let code_data =
        test_plugin_code_data(suite.test_contracts.pre_tx.0, suite.test_contracts.pre_tx.1);

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
