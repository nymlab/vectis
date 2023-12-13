use cosmwasm_std::{coin, to_binary, CosmosMsg, WasmMsg};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{
    interface::{registry_management_trait, wallet_plugin_trait},
    types::plugin::{PluginListResponse, PluginMigrateParams, PluginPermission, PluginSource},
    types::wallet::TestMigrateMsg,
};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            vectis_committee,
            wallet::{add_test_plugin, create_webauthn_wallet, sign_and_submit},
        },
    },
};

#[test]
#[serial]
fn can_migrate_plugin() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    vectis_committee::execute(
        suite.app,
        suite.deployer.clone(),
        suite.plugin_registry.clone(),
        &registry_management_trait::sv::ExecMsg::RegisterPlugin {
            code_data: test_plugin_code_data(
                suite.test_contracts.pre_tx.0,
                suite.test_contracts.pre_tx.1,
            ),
            metadata_data: test_plugin_metadata(),
        },
        &[coin(REGISTRY_FEE, "uosmo")],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    let vid = "test-user";
    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    add_test_plugin(
        &app,
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
        1,
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let plugins: PluginListResponse = wallet
        .query(&wallet_plugin_trait::sv::QueryMsg::Plugins {
            ty: PluginPermission::PreTxCheck,
            start_after: None,
            limit: None,
        })
        .unwrap();

    let mut new_code_data = test_plugin_code_data(
        suite.test_contracts.post_tx.0,
        suite.test_contracts.post_tx.1,
    );
    new_code_data.latest_contract_version = "new-version".into();

    vectis_committee::execute(
        &app,
        suite.deployer,
        suite.plugin_registry.clone(),
        &registry_management_trait::sv::ExecMsg::NewPluginVersion {
            id: 1,
            code_update: Some(new_code_data.clone()),
            metadata_update: test_plugin_metadata(),
        },
        &[],
        &suite.accounts[ICOMMITTEE],
    )
    .unwrap();

    sign_and_submit(
        &app,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&wallet_plugin_trait::sv::ExecMsg::update_plugins(vec![
                PluginMigrateParams {
                    plugin_addr: plugins.plugins[0].clone().0,
                    plugin_permission: PluginPermission::PreTxCheck,
                    target_src: PluginSource::VectisRegistry(1, Some("new-version".into())),
                    migration_msg: to_binary(&test_vectis_pre_tx::contract::sv::MigrateMsg {
                        msg: TestMigrateMsg {
                            name: "NEW".into(),
                            version: "new-version".into(),
                        },
                    })
                    .unwrap(),
                    funds: vec![],
                },
            ]))
            .unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();
}
