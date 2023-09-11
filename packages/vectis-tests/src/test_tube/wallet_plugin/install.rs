use cosmwasm_std::{coin, to_binary, CosmosMsg, Empty, WasmMsg};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{
    interface::{
        registry_management_trait, registry_service_trait,
        wallet_plugin_trait::ExecMsg as WalletPluginExecMsg,
        wallet_trait::ExecMsg as WalletExecMsg,
    },
    types::{
        authenticator::EmptyInstantiateMsg,
        plugin::{PluginInstallParams, PluginPermission, PluginSource, PluginsResponse},
        plugin_registry::{Subscriber, SubscriptionTier},
    },
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{
                add_test_plugin, create_webauthn_wallet, sign_and_create_relay_tx, sign_and_submit,
            },
        },
    },
};

#[test]
#[serial]
fn install_plugin_shows_in_proxy_and_registry() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let registry = Contract::from_addr(&app, suite.plugin_registry);

    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();
    let first_plugin = plugins.plugins[0].clone();

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(first_plugin.id, None),
            permission: PluginPermission::PreTxCheck,
            label: "plugin_install".into(),
            funds: vec![],
            instantiate_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
        }],
    };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    let sub_result: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();
    assert!(sub_result.is_some());
    let subscriber = sub_result.unwrap();

    assert_eq!(subscriber.tier, SubscriptionTier::Free);
    assert_eq!(subscriber.plugin_installed.len(), 1);
    assert!(subscriber.plugin_installed.contains(&first_plugin.id));
}

#[test]
#[serial]
fn wrong_controller_cannot_install() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";
    let other_vid = "other_test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let (_, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        other_vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(1, None),
            permission: PluginPermission::PreTxCheck,
            label: "plugin_install".into(),
            funds: vec![],
            instantiate_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
        }],
    };

    // other vid signs for vid
    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        0,
        other_vid,
    );

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    wallet
        .execute(
            &WalletExecMsg::AuthExec {
                transaction: relay_tx,
            },
            &[],
            &suite.accounts[IRELAYER],
        )
        .unwrap_err();
}

#[test]
#[serial]
fn cannot_install_same_plugin_twice() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(1, None),
            permission: PluginPermission::PreTxCheck,
            label: "plugin_install".into(),
            funds: vec![],
            instantiate_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
        }],
    };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    // install the second time should fail
    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap_err();
}

#[test]
#[serial]
fn cannot_install_more_than_free_limit_until_subscribe() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    // Add plugins to the max allowed for free tier
    // we are adding 1 that we know is pre-tx check
    let max_plugins = tier_0().max_plugins;
    for i in 1..max_plugins + 1 {
        add_test_plugin(
            &app,
            vid,
            wallet_addr.as_str(),
            &suite.accounts[IRELAYER],
            (i) as u64,
        )
        .unwrap();
    }

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let subscription: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();

    assert_eq!(
        subscription.unwrap().plugin_installed.len(),
        max_plugins as usize
    );

    // try to add one more should fail
    add_test_plugin(
        &app,
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
        (max_plugins + 1) as u64,
    )
    .unwrap_err();

    sign_and_submit(
        &app,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: suite.plugin_registry.clone(),
            msg: to_binary(&registry_service_trait::ExecMsg::subscribe(
                SubscriptionTier::L1,
            ))
            .unwrap(),
            funds: vec![coin(TIER_1_FEE, DENOM)],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    // try again should succeed
    add_test_plugin(
        &app,
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
        (max_plugins + 1) as u64,
    )
    .unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let subscription: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();

    assert_eq!(
        subscription.unwrap().plugin_installed.len(),
        (max_plugins + 1) as usize
    )
}
