use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, vectis_committee, wallet::default_entity},
    },
};
use cosmwasm_std::{coin, to_binary, Addr, Binary, CanonicalAddr, HexBinary};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Bank, OsmosisTestApp};
use std::collections::BTreeMap;
use test_tube::module::Module;
use vectis_wallet::{
    interface::registry_management_trait,
    types::{
        plugin::{Plugin, PluginsResponse, VersionDetails},
        plugin_registry::{SubscriptionTier, TierDetails},
    },
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
            code_data: test_plugin_code_data(suite.test_plugin_code_id),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    );

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

    let code_data = test_plugin_code_data(suite.test_plugin_code_id);
    let expected_plugin = Plugin {
        id: 1,
        creator: canonical_valid_osmo(),
        display_name: test_plugin_metadata().display_name,
        latest_contract_version: code_data.latest_contract_version.clone(),
        versions: BTreeMap::from([(
            code_data.latest_contract_version,
            VersionDetails {
                code_id: suite.test_plugin_code_id,
                code_hash: code_data.new_code_hash,
                ipfs_hash: test_plugin_metadata().ipfs_hash,
            },
        )]),
    };

    assert_eq!(plugins.plugins[0], expected_plugin);
    assert_eq!(plugin, expected_plugin)
}

#[test]
#[should_panic]
fn cannot_register_without_correct_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::RegisterPlugin {
            code_data: test_plugin_code_data(suite.test_plugin_code_id),
            metadata_data: test_plugin_metadata(),
        },
        // INCORRECT fee
        &[coin(WALLET_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    );
}

#[test]
fn not_deployer_cannot_register() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let registry = Contract::from_addr(&app, suite.plugin_registry);

    // Sent from another signer
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
