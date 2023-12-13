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

#[test]
fn register_plugin_works() {
    let suite = VectisTestSuite::new();

    let plugin_code_id = TestPreTxPluginCodeId::store_code(&suite.app).code_id();

    let code_hash = "963fe5826605ac76f46f2ace791c8c07cefba227555cb774a5a4757e67cabdbe";
    let code_data = test_plugin_code_data(plugin_code_id, &code_hash);
    let metadata_data = test_plugin_metadata();

    let registry = VectisPluginRegistryProxy::new(suite.plugin_registry, &suite.app);

    registry
        .registry_management_trait_proxy()
        .register_plugin(code_data.clone(), metadata_data.clone())
        .with_funds(&[coin(REGISTRY_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    let mut plugins = registry
        .registry_management_trait_proxy()
        .get_plugins(None, None)
        .unwrap();

    assert_eq!(plugins.plugins.len(), 1);
    let plugin = plugins.plugins.pop().unwrap();
    assert_eq!(
        plugin.versions.get(VECTIS_VERSION).unwrap().code_id,
        plugin_code_id
    )
}

#[test]
fn register_plugin_with_same_code_id_fails() {
    let suite = VectisTestSuite::new();

    let plugin_code_id = TestPreTxPluginCodeId::store_code(&suite.app).code_id();

    let code_hash = "963fe5826605ac76f46f2ace791c8c07cefba227555cb774a5a4757e67cabdbe";
    let code_data = test_plugin_code_data(plugin_code_id, &code_hash);
    let metadata_data = test_plugin_metadata();

    let registry = VectisPluginRegistryProxy::new(suite.plugin_registry, &suite.app);

    registry
        .registry_management_trait_proxy()
        .register_plugin(code_data.clone(), metadata_data.clone())
        .with_funds(&[coin(REGISTRY_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    // Try again should fail
    let err = registry
        .registry_management_trait_proxy()
        .register_plugin(code_data, metadata_data)
        .with_funds(&[coin(REGISTRY_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap_err();

    assert_eq!(err, PluginRegError::CodeIdAlreadyRegistered)
}

#[test]
fn new_plugin_version_with_same_code_id_fails() {
    let suite = VectisTestSuite::new();

    let plugin_code_id = TestPreTxPluginCodeId::store_code(&suite.app).code_id();

    let code_hash = "963fe5826605ac76f46f2ace791c8c07cefba227555cb774a5a4757e67cabdbe";
    let code_data = test_plugin_code_data(plugin_code_id, &code_hash);
    let metadata_data = test_plugin_metadata();

    let registry = VectisPluginRegistryProxy::new(suite.plugin_registry, &suite.app);

    registry
        .registry_management_trait_proxy()
        .register_plugin(code_data.clone(), metadata_data.clone())
        .with_funds(&[coin(REGISTRY_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    // Try again should fail
    let err = registry
        .registry_management_trait_proxy()
        .new_plugin_version(1, Some(code_data), metadata_data)
        .call(suite.deployer.as_str())
        .unwrap_err();

    assert_eq!(err, PluginRegError::CodeIdAlreadyRegistered)
}
