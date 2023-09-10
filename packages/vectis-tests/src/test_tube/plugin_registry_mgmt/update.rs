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
fn can_update_with_correct_params() {
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


}

#[test
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
