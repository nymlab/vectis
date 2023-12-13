use vectis_plugin_registry::service::sv::test_utils::RegistryServiceTrait;
use vectis_wallet::types::error::PluginRegError;

use crate::unit_tests::utils::*;
use serial_test::serial;

#[test]
#[ignore]
fn expired_subscriptions_cannot_exec_plugins() {}

#[test]
#[ignore]
fn removed_plugins_cannot_execute() {}

#[test]
#[ignore]
fn controller_can_upgrade_tiers() {}

#[test]
#[ignore]
fn controller_can_downgrade_tiers() {}

#[test]
#[serial]
fn proxy_wallet_can_subscribe() {
    let suite = VectisTestSuite::new();
    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into(), vec![]);

    let subscription_msg = RegistryServiceExecMsg::Subscribe {
        tier: SubscriptionTier::Free,
    };

    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);
    let info = wallet.wallet_trait_proxy().info().unwrap();
    let msg = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: suite.plugin_registry.to_string(),
            msg: to_binary(&subscription_msg).unwrap(),
            funds: vec![],
        })],
        info.controller.nonce,
        "vectis-wallet",
    );
    wallet
        .wallet_trait_proxy()
        .auth_exec(msg)
        .call(suite.deployer.as_str())
        .unwrap();

    let registry = VectisPluginRegistryProxy::new(suite.plugin_registry.clone(), &suite.app);
    let subscription_detail = registry
        .registry_service_trait_proxy()
        .subsciption_details(wallet_addr.to_string())
        .unwrap()
        .unwrap();

    assert_eq!(subscription_detail.tier, SubscriptionTier::Free)
}

#[test]
fn not_supported_code_id_wallet_cannot_subscribe() {
    let suite = VectisTestSuite::new();

    let registry = VectisPluginRegistryProxy::new(suite.plugin_registry.clone(), &suite.app);

    let err = registry
        .registry_service_trait_proxy()
        .subscribe(SubscriptionTier::Free)
        .call(suite.deployer.as_str())
        .unwrap_err();

    assert_eq!(err, PluginRegError::UnsupportedWallet)
}
