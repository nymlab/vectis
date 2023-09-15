use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, vectis_committee},
    },
};
use cosmwasm_std::coin;
use osmosis_test_tube::OsmosisTestApp;
use vectis_wallet::{interface::registry_management_trait, types::plugin::Plugin};

#[test]
fn can_update_correct_code_data() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let code_data = test_plugin_code_data(suite.test_plugins.pre_tx.0, suite.test_plugins.pre_tx.1);
    let mut new_code_data =
        test_plugin_code_data(suite.test_plugins.post_tx.0, suite.test_plugins.post_tx.1);
    new_code_data.latest_contract_version = "new-version".into();

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

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::NewPluginVersion {
            id: 1,
            code_update: Some(new_code_data.clone()),
            metadata_update: test_plugin_metadata(),
        },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let plugin: Plugin = registry
        .query(&registry_management_trait::QueryMsg::GetPluginById { id: 1 })
        .unwrap();

    assert_eq!(
        plugin.latest_contract_version,
        new_code_data.latest_contract_version.clone()
    );

    let new_version_details = plugin
        .get_version_details(&new_code_data.latest_contract_version.clone())
        .unwrap();
    assert_eq!(new_version_details.code_id, new_code_data.new_code_id);
    assert_eq!(new_version_details.code_hash, new_code_data.new_code_hash)
}
