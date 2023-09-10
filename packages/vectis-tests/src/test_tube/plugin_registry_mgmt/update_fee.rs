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
    interface::registry_management_trait, types::plugin_registry::RegistryConfigResponse,
};

#[test]
fn deployer_can_update_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    let config: RegistryConfigResponse = registry
        .query(&registry_management_trait::QueryMsg::GetConfig {})
        .unwrap();

    assert_eq!(config.registry_fee, coin(REGISTRY_FEE, DENOM));

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::UpdateRegistryFee {
            new_fee: coin(REGISTRY_FEE + 10, DENOM),
        },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let config: RegistryConfigResponse = registry
        .query(&registry_management_trait::QueryMsg::GetConfig {})
        .unwrap();

    assert_eq!(config.registry_fee, coin(REGISTRY_FEE + 10, DENOM));

    // old_fee cannot add plugin
    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry,
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
fn not_deployer_cannot_update_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    // Sent from another signer
    registry
        .execute(
            &registry_management_trait::ExecMsg::UpdateRegistryFee {
                new_fee: coin(REGISTRY_FEE + 10, DENOM),
            },
            &[],
            &suite.accounts[ICOMMITTEE],
        )
        .unwrap_err();
}
