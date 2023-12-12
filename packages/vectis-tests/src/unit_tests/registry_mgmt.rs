use vectis_wallet::types::error::PluginRegError;

use crate::unit_tests::utils::*;

#[test]
fn instantiate_registry_correctly() {
    let app = App::default();
    let deployer = "deployer";
    let registry_code_id =
        vectis_plugin_registry::contract::sv::multitest_utils::CodeId::store_code(&app);
    let subscription_tier = vec![(SubscriptionTier::Free, tier_0())];

    let registry = registry_code_id
        .instantiate(coin(REGISTRY_FEE, DENOM), subscription_tier.clone())
        .with_label("Vectis Plugin Registry")
        .call(deployer)
        .unwrap();

    let config = registry
        .registry_management_trait_proxy()
        .get_config()
        .unwrap();

    assert_eq!(config.registry_fee, coin(REGISTRY_FEE, DENOM));
    assert_eq!(config.deployer_addr, deployer);
    assert_eq!(
        config.subscription_tiers,
        subscription_tier
            .into_iter()
            .map(|(tier, details)| (tier as u8, details))
            .collect::<Vec<(u8, TierDetails)>>()
    );
}

#[test]
fn instantiate_error_on_duplicated_subscription_tier() {
    let app = App::default();
    let deployer = "deployer";
    let registry_code_id =
        vectis_plugin_registry::contract::sv::multitest_utils::CodeId::store_code(&app);
    let subscription_tier = vec![
        (SubscriptionTier::Free, tier_0()),
        (SubscriptionTier::Free, tier_0()),
    ];

    let err = registry_code_id
        .instantiate(coin(REGISTRY_FEE, DENOM), subscription_tier.clone())
        .with_label("Vectis Plugin Registry")
        .call(deployer)
        .unwrap_err();

    assert_eq!(err, PluginRegError::TierExists)
}
