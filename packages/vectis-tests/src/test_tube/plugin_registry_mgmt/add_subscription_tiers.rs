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
    interface::registry_management_trait::sv as registry_management_trait,
    types::plugin_registry::RegistryConfigResponse,
    types::plugin_registry::{SubscriptionTier, TierDetails},
};

#[test]
fn deployer_can_add_subscription_tier() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    let config: RegistryConfigResponse = registry
        .query(&registry_management_trait::QueryMsg::GetConfig {})
        .unwrap();

    let tiers_len = config.subscription_tiers.len();

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::AddOrUpdateSubscriptionTiers {
            tier: SubscriptionTier::Other as u8,
            details: TierDetails {
                max_plugins: 3,
                duration: None,
                fee: coin(TIER_1_FEE, DENOM),
            },
        },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let config: RegistryConfigResponse = registry
        .query(&registry_management_trait::QueryMsg::GetConfig {})
        .unwrap();

    assert_eq!(config.subscription_tiers.len(), tiers_len + 1);
}

#[test]
fn existing_tier_cannot_be_added() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let config: RegistryConfigResponse = registry
        .query(&registry_management_trait::QueryMsg::GetConfig {})
        .unwrap();
    assert_eq!(config.subscription_tiers.len(), 2);
    let (current_tier, _) = config.subscription_tiers[0];

    vectis_committee::execute(
        &app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::ExecMsg::AddOrUpdateSubscriptionTiers {
            tier: current_tier,
            details: TierDetails {
                max_plugins: 3,
                duration: None,
                fee: coin(TIER_1_FEE, DENOM),
            },
        },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap_err();
}

#[test]
fn not_deployer_cannot_add_subscription_tier() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());

    // Sent from another signer
    registry
        .execute(
            &registry_management_trait::ExecMsg::AddOrUpdateSubscriptionTiers {
                tier: SubscriptionTier::L1 as u8,
                details: TierDetails {
                    max_plugins: 3,
                    duration: None,
                    fee: coin(TIER_1_FEE, DENOM),
                },
            },
            &[],
            &suite.accounts[ICOMMITTEE],
        )
        .unwrap_err();
}
