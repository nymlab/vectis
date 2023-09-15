use sylvia::multitest::App;

use crate::constants::*;
use cosmwasm_std::coin;
use vectis_plugin_registry::management::contract::test_utils::RegistryManagementTrait;
use vectis_wallet::types::plugin_registry::{SubscriptionTier, TierDetails};

#[test]
fn instantiate_registry_correctly() {
    let app = App::default();
    let deployer = "deployer";
    let registry_code_id =
        vectis_plugin_registry::contract::multitest_utils::CodeId::store_code(&app);
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
