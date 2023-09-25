use cosmwasm_std::{to_binary, CosmosMsg, Empty};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{
    interface::{
        registry_service_trait,
        wallet_plugin_trait::{self, ExecMsg as WalletPluginExecMsg},
    },
    types::{
        authenticator::EmptyInstantiateMsg,
        plugin::{PluginInstallParams, PluginListResponse, PluginPermission, PluginSource},
        plugin_registry::Subscriber,
    },
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{create_webauthn_wallet, sign_and_submit},
        },
    },
};

#[test]
#[serial]
fn remove_installed_plugin_successfully() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);

    suite.register_plugins();

    let vid = "test-user";
    let registry = Contract::from_addr(&app, suite.plugin_registry);

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    // Account not subscribed in the registry before installing a pluging
    let sub_result: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();
    assert!(sub_result.is_none());

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(suite.test_contracts.pre_tx.2, None),
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

    // Account has now got one plugin
    let sub_result: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();
    let sub = sub_result.unwrap();
    assert_eq!(sub.plugin_installed.len(), 1);

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: PluginListResponse = wallet
        .query(&wallet_plugin_trait::QueryMsg::Plugins {})
        .unwrap();
    assert_eq!(info.pre_tx.len(), 1);

    // Remove plugin
    let remove_plugin_msg = WalletPluginExecMsg::RemovePlugins {
        plugin_addrs: info
            .pre_tx
            .iter()
            .map(|(addr, _)| addr.to_string())
            .collect(),
    };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&remove_plugin_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    // Check wallet and registry that this was removed
    let info: PluginListResponse = wallet
        .query(&wallet_plugin_trait::QueryMsg::Plugins {})
        .unwrap();
    assert_eq!(info.pre_tx.len(), 0);

    let sub_result: Option<Subscriber> = registry
        .query(&registry_service_trait::QueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();
    let sub = sub_result.unwrap();
    assert_eq!(sub.plugin_installed.len(), 0);
}
