use crate::helpers::{sign_and_create_relay_tx, webauthn_entity};
use crate::unit_tests::utils::*;
use serial_test::serial;

#[test]
#[serial]
fn rotate_to_new_addresses_works() {
    let suite = VectisTestSuite::new();

    let entities = make_entities(4);

    let wallet_addr =
        suite.create_default_wallet(entities[0].clone(), "vectis-wallet".into(), vec![]);
    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);

    for (index, entity) in entities.iter().enumerate() {
        if index != 0 {
            let info = wallet.wallet_trait_proxy().info().unwrap();
            let rotate_wallet = WalletExecMsg::ControllerRotation {
                new_controller: entity.clone(),
            };
            let msg = sign_and_create_relay_tx(
                vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: wallet_addr.to_string(),
                    msg: to_binary(&rotate_wallet).unwrap(),
                    funds: vec![],
                })],
                info.controller.nonce,
                (index - 1).to_string().as_str(),
            );

            wallet
                .wallet_trait_proxy()
                .auth_exec(msg)
                .call(VALID_OSMO_ADDR)
                .unwrap();

            let info = wallet.wallet_trait_proxy().info().unwrap();
            assert_eq!(info.controller.data, entity.data);
        }
    }
}

#[test]
#[serial]
fn cannot_rotate_to_existing_address() {
    let suite = VectisTestSuite::new();

    let entities = make_entities(4);

    let wallet_addr =
        suite.create_default_wallet(entities[0].clone(), "vectis-wallet".into(), vec![]);
    let wallet = VectisProxyProxy::new(wallet_addr.clone(), &suite.app);

    for (index, entity) in entities.iter().enumerate() {
        if index != 0 {
            let info = wallet.wallet_trait_proxy().info().unwrap();
            let rotate_wallet = WalletExecMsg::ControllerRotation {
                new_controller: entity.clone(),
            };
            let msg = sign_and_create_relay_tx(
                vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
                    contract_addr: wallet_addr.to_string(),
                    msg: to_binary(&rotate_wallet).unwrap(),
                    funds: vec![],
                })],
                info.controller.nonce,
                (index - 1).to_string().as_str(),
            );

            wallet
                .wallet_trait_proxy()
                .auth_exec(msg)
                .call(VALID_OSMO_ADDR)
                .unwrap();
        }
    }

    // Create rotate to an entity in the list
    let info = wallet.wallet_trait_proxy().info().unwrap();
    let rotate_wallet = WalletExecMsg::ControllerRotation {
        new_controller: entities[0].clone(),
    };
    let msg = sign_and_create_relay_tx(
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&rotate_wallet).unwrap(),
            funds: vec![],
        })],
        info.controller.nonce,
        (3).to_string().as_str(),
    );

    wallet
        .wallet_trait_proxy()
        .auth_exec(msg)
        .call(VALID_OSMO_ADDR)
        .unwrap_err();
}

fn make_entities(total: u8) -> Vec<Entity> {
    let mut entities = vec![];
    for i in 0..total {
        let pubkey = must_create_credential(&i.to_string());
        entities.push(webauthn_entity(&pubkey));
    }
    entities
}
