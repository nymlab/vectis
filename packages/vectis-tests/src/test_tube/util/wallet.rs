use cosmwasm_std::{coin, to_binary, Addr, Binary, CosmosMsg};
use osmosis_test_tube::OsmosisTestApp;
use test_tube::SigningAccount;

use crate::{
    constants::*,
    test_tube::util::{
        contract::Contract,
        passkey::{
            de_client_data, hash_to_base64url_string, hash_to_hex_string, must_create_credential,
            must_get_credential,
        },
    },
};
use vectis_wallet::{
    interface::factory_service_trait::{
        ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
    },
    types::{
        authenticator::{Authenticator, AuthenticatorProvider, AuthenticatorType},
        entity::Entity,
        factory::CreateWalletMsg,
        wallet::{Nonce, RelayTransaction, VectisRelayedTx, WebauthnRelayedTxMsg},
    },
};

pub fn default_entity() -> Entity {
    Entity {
        auth: Authenticator {
            ty: AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: to_binary(&"data").unwrap(),
        nonce: 0,
    }
}

pub fn webauthn_entity(data: &[u8]) -> Entity {
    Entity {
        auth: Authenticator {
            ty: AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: Binary::from(data),
        nonce: 0,
    }
}

pub fn create_webauthn_wallet<'a>(
    app: &'a OsmosisTestApp,
    factory_addr: &'a str,
    vid: &'a str,
    init_balance: u128,
    signer: &'a SigningAccount,
) -> (Addr, Vec<u8>) {
    let pubkey = must_create_credential(vid);
    let entity = webauthn_entity(&pubkey);

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![coin(init_balance, DENOM)],
            vid: vid.into(),
            initial_data: vec![],
            plugins: vec![],
        },
    };

    let factory = Contract::from_addr(&app, factory_addr.into());
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE + INIT_BALANCE, DENOM)],
            signer,
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid.into() })
        .unwrap();

    (wallet_addr.unwrap(), pubkey)
}

pub fn sign_and_create_relay_tx(
    messages: Vec<CosmosMsg>,
    nonce: Nonce,
    vid: &str,
) -> RelayTransaction {
    // =======================
    // Signing data
    // =======================
    let signed_msg = VectisRelayedTx {
        messages,
        nonce,
        sponsor_fee: None,
    };
    let signed_msg_str = serde_json_wasm::to_string(&signed_msg).unwrap();

    // We encode the hash vec into hex to pass to the cli
    let challenge = hash_to_hex_string(signed_msg_str.as_bytes());
    let response = must_get_credential(vid, challenge.clone());

    // Compare the expeted challenge value to the one returned from passkey
    let expect_challenge = hash_to_base64url_string(signed_msg_str.as_bytes());
    let challenge_in_client_data = de_client_data(&response.client_data_json);
    assert_eq!(expect_challenge, challenge_in_client_data.challenge);

    RelayTransaction {
        message: to_binary(&WebauthnRelayedTxMsg {
            signed_data: signed_msg_str,
            auth_data: Binary::from(response.authenticator_data),
            client_data: Binary::from(response.client_data_json),
        })
        .unwrap(),
        signature: Binary::from(response.signature),
    }
}
