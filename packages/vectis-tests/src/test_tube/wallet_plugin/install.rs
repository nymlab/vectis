use cosmwasm_std::{coin, to_binary, BankMsg, CosmosMsg, Empty};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Account, Bank, OsmosisTestApp};
use serial_test::serial;
use test_tube::module::Module;

use vectis_wallet::{
    interface::{
        registry_management_trait, registry_service_trait,
        wallet_plugin_trait::ExecMsg as WalletPluginExecMsg,
        wallet_trait::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    },
    types::{
        authenticator::EmptyInstantiateMsg,
        plugin::{PluginInstallParams, PluginPermission, PluginSource, PluginsResponse},
        plugin_registry::{Subscriber, SubscriptionTier},
        wallet::{Nonce, WalletInfo},
    },
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            msgs::simple_bank_send,
            wallet::{create_webauthn_wallet, sign_and_create_relay_tx},
        },
    },
};

#[test]
#[serial]
fn install_plugin_shows_in_proxy_and_registry() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    suite.register_plugins();

    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );
    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let registry = Contract::from_addr(&app, suite.plugin_registry);

    let plugins: PluginsResponse = registry
        .query(&registry_management_trait::QueryMsg::GetPlugins {
            limit: None,
            start_after: None,
        })
        .unwrap();
    let first_plugin = plugins.plugins[0].clone();

    // ===========================
    // Signing and create tx data
    // ===========================
    //     pub src: PluginSource,
    //pub instantiate_msg: Binary,
    //pub permission: PluginPermission,
    //pub label: String,
    //pub funds: Vec<Coin>,
    //
    println!("all plugins: {:?}", plugins);

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(first_plugin.id, None),
            permission: PluginPermission::PreTxCheck,
            label: "pre tx".into(),
            funds: vec![],
            instantiate_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
        }],
    };
    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        info.controller.nonce,
        vid,
    );

    wallet
        .execute(
            &WalletExecMsg::AuthExec {
                transaction: relay_tx,
            },
            &[],
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

    println!("subscriber: {:?}", subscriber);
    assert_eq!(subscriber.tier, SubscriptionTier::Free);
    assert_eq!(subscriber.plugin_installed.len(), 1);
    assert!(subscriber.plugin_installed.contains(&first_plugin.id));
}

#[test]
fn todo_wrong_controller_cannot_install() {}

#[test]
fn todo_cannot_install_same_plugin_twice() {}
