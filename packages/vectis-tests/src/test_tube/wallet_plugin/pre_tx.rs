use cosmwasm_std::{coin, to_binary, BankMsg, CosmosMsg, Empty, WasmMsg};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{interface::registry_service_trait, types::plugin_registry::SubscriptionTier};

use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::wallet::{add_test_plugin, create_webauthn_wallet, sign_and_submit},
    },
};

// This uses the code from contracts/test-contracts/pre-tx-checks
// It will return false if msg contains bankmsg
#[test]
#[serial]
fn pre_tx_checks() {
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

    add_test_plugin(
        &app,
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
        suite.test_contracts.pre_tx.2,
    )
    .unwrap();

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&CosmosMsg::<Empty>::Bank(BankMsg::Send {
                to_address: VALID_OSMO_ADDR.into(),
                amount: vec![coin(2, DENOM)],
            }))
            .unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap_err();

    // Should be able to send wasm msg
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
}
