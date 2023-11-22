use crate::helpers::{sign_and_create_relay_tx, webauthn_entity};
use crate::unit_tests::utils::*;
use serial_test::serial;

#[test]
#[serial]
fn test_create_wallet_with_passkey() {
    let suite = VectisTestSuite::new();

    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into());

    let wallet = VectisProxyProxy::new(wallet_addr, &suite.app);
    let info: WalletInfo = wallet.wallet_trait_proxy().info().unwrap();

    assert_eq!(info.controller.data.0, pubkey);
}

#[test]
#[serial]
fn update_data_only_from_controller() {
    let suite = VectisTestSuite::new();

    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into());

    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);

    let info = wallet.wallet_trait_proxy().info().unwrap();

    let new_data = vec![(
        to_binary("new-key").unwrap(),
        Some(to_binary("new-data").unwrap()),
    )];

    let update_data_msg = WalletExecMsg::UpdateData {
        data: new_data.clone(),
    };

    let msg = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&update_data_msg).unwrap(),
            funds: vec![],
        })],
        info.controller.nonce,
        "vectis-wallet",
    );

    wallet
        .wallet_trait_proxy()
        .auth_exec(msg)
        .call(VALID_OSMO_ADDR)
        .unwrap();

    let new_info = wallet.wallet_trait_proxy().info().unwrap();
    assert_eq!(info.controller.nonce + 1, new_info.controller.nonce);
}
