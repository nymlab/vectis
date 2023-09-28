use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{create_webauthn_wallet, sign_migration_msg},
        },
    },
};
use cosmwasm_std::{to_binary, CosmosMsg, WasmMsg};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;
use vectis_wallet::{
    interface::wallet_trait,
    types::wallet::{TestMigrateMsg, WalletInfo},
};

#[test]
#[serial]
fn controller_can_migrate_wallet() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid = "test-user-1";
    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&wallet_trait::QueryMsg::Info {}).unwrap();

    let initial_nonce = info.controller.nonce;
    assert_eq!(info.version.version, VECTIS_VERSION);
    assert_eq!(info.version.contract, "vectis-proxy");

    sign_migration_msg(
        &app,
        vec![CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: wallet_addr.to_string(),
            new_code_id: suite.test_contracts.proxy_migrate.0,
            msg: to_binary(&vectis_proxy::contract::MigrateMsg {
                msg: TestMigrateMsg {
                    name: "TESTNAME".into(),
                    version: PROXY_MIGRATE_VERSION.into(),
                },
            })
            .unwrap(),
        })],
        vid,
        wallet_addr.as_str(),
        &suite.factory,
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&wallet_trait::QueryMsg::Info {}).unwrap();

    assert_eq!(info.version.version, PROXY_MIGRATE_VERSION);
    assert_eq!(info.version.contract, "TESTNAME");
    assert_eq!(info.controller.nonce, initial_nonce + 1);
}

#[test]
#[serial]
fn not_controller_cannot_migrate_wallet() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid = "test-user-1";
    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );
    let vid_1 = "test-user-2";
    let (wallet_addr_other, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid_1,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    sign_migration_msg(
        &app,
        vec![CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: wallet_addr.to_string(),
            new_code_id: suite.test_contracts.proxy_migrate.0,
            msg: to_binary(&vectis_proxy::contract::MigrateMsg {
                msg: TestMigrateMsg {
                    name: "TESTNAME".into(),
                    version: PROXY_MIGRATE_VERSION.into(),
                },
            })
            .unwrap(),
        })],
        // Signed by vid_1
        vid_1,
        // execute through the entity this wallet
        wallet_addr_other.as_str(),
        &suite.factory,
        &suite.accounts[IRELAYER],
    )
    .unwrap_err();
}

#[test]
#[serial]
fn cannot_migrate_to_unsupported_proxies() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid = "test-user-1";
    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    sign_migration_msg(
        &app,
        vec![CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: wallet_addr.to_string(),
            new_code_id: suite.test_contracts.exec.0,
            msg: to_binary(&vectis_proxy::contract::MigrateMsg {
                msg: TestMigrateMsg {
                    name: "TESTNAME".into(),
                    version: PROXY_MIGRATE_VERSION.into(),
                },
            })
            .unwrap(),
        })],
        vid,
        wallet_addr.as_str(),
        &suite.factory,
        &suite.accounts[IRELAYER],
    )
    .unwrap_err();
}
