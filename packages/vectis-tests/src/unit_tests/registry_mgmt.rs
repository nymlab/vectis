use std::collections::BTreeMap;
use sylvia::multitest::App;

use crate::constants::*;
use cosmwasm_std::{coin, Addr, CanonicalAddr, HexBinary};
use vectis_plugin_registry::management::contract::test_utils::RegistryManagementTrait;
use vectis_wallet::types::{
    plugin::{Plugin, VersionDetails},
    plugin_registry::{SubscriptionTier, TierDetails},
};

#[test]
fn instantiate_registry_correctly() {
    let app = App::default();
    let deployer = "deployer";
    let registry_code_id =
        vectis_plugin_registry::contract::multitest_utils::CodeId::store_code(&app);
    let subscription_tier = vec![(SubscriptionTier::Free, tier_0())];
    let supported_proxies = vec![(
        HexBinary::from(PROXY_CODE_HASH.as_bytes()),
        VECTIS_VERSION.into(),
    )];

    let registry = registry_code_id
        .instantiate(
            coin(REGISTRY_FEE, DENOM),
            //subscription_tiers: Vec<(SubscriptionTier, TierDetails)>,
            //supported_proxies: Vec<(HexBinary, String)>)
            subscription_tier.clone(),
            supported_proxies.clone(),
        )
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
        config.supported_proxies,
        supported_proxies
            .into_iter()
            .map(|(hash, version)| (hash.to_hex(), version))
            .collect::<Vec<(String, String)>>()
    );
    assert_eq!(
        config.subscription_tiers,
        subscription_tier
            .into_iter()
            .map(|(tier, details)| (tier as u8, details))
            .collect::<Vec<(u8, TierDetails)>>()
    );
}

//#[test]
//fn deployer_can_register_plugins_with_correct_fee() {
//    let deployer = "deployer";
//
//    let app = App::<cw_multi_test::App>::custom(|router, _, storage| {
//        router
//            .bank
//            .init_balance(
//                storage,
//                &Addr::unchecked(deployer),
//                vec![coin(DEPLOYER_INIT_BALANCE, DENOM)],
//            )
//            .unwrap();
//    });
//
//    let registry_code_id =
//        vectis_plugin_registry::contract::multitest_utils::CodeId::store_code(&app);
//    let subscription_tier = vec![(SubscriptionTier::Free, tier_0())];
//    let supported_proxies = vec![(
//        HexBinary::from(PROXY_CODE_HASH.as_bytes()),
//        VECTIS_VERSION.into(),
//    )];
//
//    let registry = registry_code_id
//        .instantiate(
//            coin(REGISTRY_FEE, DENOM),
//            //subscription_tiers: Vec<(SubscriptionTier, TierDetails)>,
//            //supported_proxies: Vec<(HexBinary, String)>)
//            subscription_tier.clone(),
//            supported_proxies.clone(),
//        )
//        .with_label("Vectis Plugin Registry")
//        .call(deployer)
//        .unwrap();
//
//    registry
//        .registry_management_trait_proxy()
//        .register_plugin(dummy_plugin_code_data(1), dummy_plugin_metadata())
//        .with_funds(&[coin(REGISTRY_FEE, DENOM)])
//        .call(deployer)
//        .unwrap();
//
//    let plugins_res = registry
//        .registry_management_trait_proxy()
//        .get_plugins(None, None)
//        .unwrap();
//    assert_eq!(plugins_res.total, 0);
//    assert_eq!(plugins_res.current_plugin_id, 1);
//
//    let plugin = registry
//        .registry_management_trait_proxy()
//        .get_plugin_by_id(0)
//        .unwrap();
//
//    let expected_plugin = Plugin {
//        id: 0,
//        creator: CanonicalAddr::from(dummy_plugin_metadata().creator.as_bytes()),
//        display_name: dummy_plugin_metadata().display_name,
//        latest_contract_version: dummy_plugin_code_data(1).latest_contract_version,
//        versions: BTreeMap::from([(
//            dummy_plugin_code_data(1).latest_contract_version,
//            VersionDetails {
//                code_id: 1,
//                code_hash: dummy_plugin_code_data(1).new_code_hash,
//                ipfs_hash: dummy_plugin_metadata().ipfs_hash,
//            },
//        )]),
//    };
//
//    assert_eq!(plugins_res.plugins[0], expected_plugin);
//    assert_eq!(plugin.unwrap(), expected_plugin)
//
//    ///// Identifier of the plugin, does not change over time
//    //pub id: u64,
//    ///// Reference Addr onchain to the creator
//    //pub creator: CanonicalAddr,
//    ///// Display name, creator can define this
//    //pub display_name: String,
//    ///// Latest cw2 contract version
//    //pub latest_contract_version: String,
//    ///// Mapping of all versions to the details
//    //pub versions: BTreeMap<String, VersionDetails>,
//}

//#[test]
//fn no_reviewer_cannot_unregister_plugins() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let err = suite
//        .unregister_plugin(&suite.controller.clone(), 1)
//        .unwrap_err();
//
//    assert_eq!(err, PRegistryContractError::Unauthorized);
//}
//
//#[test]
//fn no_reviewer_cannot_update_plugins() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let err = suite
//        .update_plugin(
//            &suite.controller.clone(),
//            1,
//            None,
//            None,
//            None,
//            None,
//            "v-new-version".into(),
//        )
//        .unwrap_err();
//
//    assert_eq!(err, PRegistryContractError::Unauthorized);
//}
//
//#[test]
//fn no_deployer_cannot_update_registry_fee() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let err = suite
//        .update_registry_fee(&suite.controller.clone(), coin(100_000, DENOM))
//        .unwrap_err();
//
//    assert_eq!(err, PRegistryContractError::Unauthorized);
//}
//
//#[test]
//fn can_update_registry_fees_to_zero_registration_fee_works() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    suite
//        .update_registry_fee(&suite.deployer.clone(), coin(0, DENOM))
//        .unwrap();
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &[])
//        .unwrap();
//
//    let resp = suite.query_registered_plugins(None, None).unwrap();
//
//    // check there is a plugin
//    assert_eq!(resp.total, 1);
//}
//
//#[test]
//fn can_update_install_fees() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    suite
//        .update_install_fee(&suite.deployer.clone(), coin(0, DENOM))
//        .unwrap();
//}
//#[test]
//fn not_deployer_cannot_update_deployer_addr() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let err = suite
//        .update_deployer_addr(&suite.controller.clone(), "test")
//        .unwrap_err();
//
//    assert_eq!(err, PRegistryContractError::Unauthorized);
//}
//
//#[test]
//fn reviewer_should_be_able_to_register_plugins() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let deployer_previous_balance = suite.query_balance(&suite.deployer.clone()).unwrap();
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//
//    let deployer_current_balance = suite.query_balance(&suite.deployer.clone()).unwrap();
//    let resp = suite.query_registered_plugins(None, None).unwrap();
//
//    // check there is a plugin
//    assert_eq!(resp.total, 1);
//
//    // check the deployer received the register fee;
//    assert_eq!(
//        deployer_current_balance.amount,
//        deployer_previous_balance
//            .amount
//            .checked_add(Uint128::from(REGISTRY_FEE))
//            .unwrap()
//    )
//}
//
//#[test]
//fn reviewer_should_be_able_to_unregister_plugins() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//
//    let resp = suite.query_registered_plugins(None, None).unwrap();
//
//    assert_eq!(resp.total, 1);
//
//    suite
//        .unregister_plugin(&suite.plugin_committee.clone(), 1)
//        .unwrap();
//
//    let resp = suite.query_registered_plugins(None, None).unwrap();
//
//    assert_eq!(resp.total, 0);
//    assert_eq!(resp.current_plugin_id, 1);
//}
//
//#[test]
//fn after_unregister_current_id_does_not_change() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    // should have 2 plugins
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//    let resp = suite.query_registered_plugins(None, None).unwrap();
//    assert_eq!(resp.total, 1);
//    suite
//        .unregister_plugin(&suite.plugin_committee.clone(), 1)
//        .unwrap();
//    let mut resp = suite.query_registered_plugins(None, None).unwrap();
//    assert_eq!(resp.total, 0);
//    assert_eq!(resp.current_plugin_id, 1);
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//    resp = suite.query_registered_plugins(None, None).unwrap();
//    assert_eq!(resp.total, 1);
//    assert_eq!(resp.plugins[0].id, 2)
//}
//
//#[test]
//fn reviewer_should_be_able_to_update_plugins() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//
//    let plugin = suite.query_plugin(1).unwrap().unwrap();
//
//    let new_code_id = 2;
//    let new_creator = "creator";
//    let new_ipfs_hash = "new_ipfs_hash";
//    let new_checksum = "new_checksum";
//    let new_version = "new_version";
//
//    suite
//        .update_plugin(
//            &suite.plugin_committee.clone(),
//            plugin.id,
//            Some(new_code_id),
//            Some(new_creator.to_string()),
//            Some(new_ipfs_hash.to_string()),
//            Some(new_checksum.to_string()),
//            new_version.to_string(),
//        )
//        .unwrap();
//
//    let plugin_after = suite.query_plugin(1).unwrap().unwrap();
//
//    let plugin_version_details = plugin.versions.get(&plugin.latest_version).unwrap();
//    assert_ne!(plugin_version_details.ipfs_hash, new_ipfs_hash);
//    assert_ne!(plugin_version_details.checksum, new_checksum);
//    assert_ne!(plugin_version_details.code_id, new_code_id);
//    assert_ne!(plugin.latest_version, new_version);
//
//    let new_plugin_version_details = plugin_after
//        .versions
//        .get(&plugin_after.latest_version)
//        .unwrap();
//    assert_eq!(new_plugin_version_details.code_id, new_code_id);
//    assert_eq!(new_plugin_version_details.ipfs_hash, new_ipfs_hash);
//    assert_eq!(new_plugin_version_details.checksum, new_checksum);
//    assert_eq!(plugin_after.latest_version, new_version);
//}
//
//#[test]
//fn reviewer_should_not_be_able_to_update_plugins_same_version_to_overwrite() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    suite
//        .register_plugin_mocked(&suite.plugin_committee.clone(), &coins(REGISTRY_FEE, DENOM))
//        .unwrap();
//
//    let plugin = suite.query_plugin(1).unwrap().unwrap();
//
//    let new_code_id = 2;
//    let new_creator = "creator";
//    let new_ipfs_hash = "new_ipfs_hash";
//    let new_checksum = "new_checksum";
//    let old_version = "0.0.1";
//
//    suite
//        .update_plugin(
//            &suite.plugin_committee.clone(),
//            plugin.id,
//            Some(new_code_id),
//            Some(new_creator.to_string()),
//            Some(new_ipfs_hash.to_string()),
//            Some(new_checksum.to_string()),
//            old_version.to_string(),
//        )
//        .unwrap_err();
//}
//
//#[test]
//fn deployer_should_be_able_to_update_registry_fee() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let new_registry_fee = coin(100_000, DENOM);
//
//    suite
//        .update_registry_fee(&suite.deployer.clone(), new_registry_fee.clone())
//        .unwrap();
//
//    let config = suite.query_config().unwrap();
//
//    assert_eq!(config.registry_fee, new_registry_fee);
//}
//
//#[test]
//fn deployer_should_be_able_to_update_deployer_addr() {
//    let mut suite = HubChainSuite::init().unwrap();
//
//    let new_deployer_addr = "new_deployer_addr";
//
//    suite
//        .update_deployer_addr(&suite.deployer.clone(), new_deployer_addr)
//        .unwrap();
//
//    let config = suite.query_config().unwrap();
//
//    assert_eq!(config.deployer_addr, new_deployer_addr);
//}
