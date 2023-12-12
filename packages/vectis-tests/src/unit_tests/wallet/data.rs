use crate::helpers::{sign_and_create_relay_tx, webauthn_entity};
use crate::unit_tests::utils::*;
use serial_test::serial;

#[test]
#[serial]
fn update_data_only_from_controller() {
    let suite = VectisTestSuite::new();

    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into(), vec![]);

    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);

    // expected new data to be added
    let new_data = vec![(
        to_binary("new-key").unwrap(),
        Some(to_binary("new-data").unwrap()),
    )];
    let update_data_msg = WalletExecMsg::UpdateData {
        data: new_data.clone(),
    };

    let info = wallet.wallet_trait_proxy().info().unwrap();
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

    let key = new_data.clone().pop().unwrap();
    let data = wallet
        .wallet_trait_proxy()
        .data(key.0.clone())
        .unwrap()
        .unwrap();
    assert_eq!(data, key.1.unwrap());

    // Another credential cannot sign
    let _ = must_create_credential("not-vectis-wallet");
    let msg_2 = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&update_data_msg).unwrap(),
            funds: vec![],
        })],
        new_info.controller.nonce,
        "not-vectis-wallet",
    );

    wallet
        .wallet_trait_proxy()
        .auth_exec(msg_2)
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    // Controller can remove data
    let new_data = vec![(to_binary("new-key").unwrap(), None)];
    let update_data_msg = WalletExecMsg::UpdateData {
        data: new_data.clone(),
    };

    let info = wallet.wallet_trait_proxy().info().unwrap();
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

    let data = wallet.wallet_trait_proxy().data(key.0).unwrap();
    assert!(data.is_none());
}

#[test]
#[serial]
fn init_data_works() {
    let suite = VectisTestSuite::new();

    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let new_data = vec![(
        to_binary("new-key").unwrap(),
        to_binary("new-data").unwrap(),
    )];
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into(), new_data.clone());

    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);
    let key = new_data.clone().pop().unwrap();
    let data = wallet.wallet_trait_proxy().data(key.0).unwrap().unwrap();
    assert_eq!(data, key.1);
}
