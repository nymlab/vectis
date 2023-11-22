use cosmwasm_std::{coin, to_binary, BankMsg, CosmosMsg, Empty};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::{interface::wallet_trait, types::wallet::WalletInfo};

use crate::{
    constants::*,
    helpers::webauthn_entity,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            passkey::must_create_credential,
            wallet::{create_webauthn_wallet, sign_and_submit},
        },
    },
};

#[test]
#[serial]
fn controller_can_rotate_to_new_controlle() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let vid = "test-user";
    let new_vid = "test-user-1";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    // Create new entity to rotate to
    let pubkey = must_create_credential(new_vid);
    let entity = webauthn_entity(&pubkey);

    let rotate_msg = wallet_trait::sv::ExecMsg::ControllerRotation {
        new_controller: entity,
    };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&rotate_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&wallet_trait::sv::QueryMsg::Info {}).unwrap();

    assert_eq!(info.controller.data, pubkey);

    // old key cannot sign
    sign_and_submit(
        &app,
        vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: VALID_OSMO_ADDR.into(),
            amount: vec![coin(1, DENOM)],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap_err();

    // new key can sign
    sign_and_submit(
        &app,
        vec![CosmosMsg::Bank(BankMsg::Send {
            to_address: VALID_OSMO_ADDR.into(),
            amount: vec![coin(1, DENOM)],
        })],
        new_vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();
}
