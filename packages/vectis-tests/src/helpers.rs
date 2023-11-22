use cosmwasm_std::{to_binary, Binary, CosmosMsg};
pub use vectis_wallet::{
    interface::wallet_trait::sv::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    types::{
        authenticator::{
            Authenticator, AuthenticatorProvider, AuthenticatorType, EmptyInstantiateMsg,
        },
        entity::Entity,
        factory::{
            AuthenticatorInstInfo, ChainConnection, CodeIdType, CreateWalletMsg, FeeType,
            FeesResponse, WalletFactoryInstantiateMsg,
        },
        plugin_registry::{SubscriptionTier, TierDetails},
        state::VectisActors,
        wallet::{Nonce, RelayTransaction, VectisRelayedTx, WebauthnRelayedTxMsg},
    },
};

use crate::passkey::*;

pub fn sign_and_create_relay_tx(
    messages: Vec<CosmosMsg>,
    nonce: Nonce,
    vid: &str,
) -> RelayTransaction {
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
