use cosmwasm_std::{from_binary, Addr, CosmosMsg, Deps, StdResult};
use sha2::{Digest, Sha256};

use crate::{
    interface::authenticator_trait,
    types::{
        authenticator::AuthenticatorType,
        entity::Entity,
        error::RelayTxError,
        wallet::{Nonce, RelayTransaction, VectisRelayedTx, WebauthnRelayedTxMsg},
    },
};

pub fn verify_cosmos_sign(
    deps: Deps,
    transaction: &RelayTransaction,
    pubkey: &[u8],
) -> StdResult<bool> {
    let hash = Sha256::digest(&transaction.message);
    let result = deps
        .api
        .secp256k1_verify(hash.as_ref(), &transaction.signature, &pubkey)?;
    Ok(result)
}

pub(crate) fn check_cosmos_relayed_tx(
    msg: &VectisRelayedTx,
    nonce: Nonce,
) -> Result<(), RelayTxError> {
    if msg.nonce != nonce {
        Err(RelayTxError::NoncesAreNotEqual)
    } else if msg.messages.is_empty() {
        Err(RelayTxError::EmptyMsg)
    } else {
        Ok(())
    }
}

pub fn relay_tx_auth_check(
    deps: Deps,
    controller: Entity,
    transaction: RelayTransaction,
    auth_addr: Option<Addr>,
) -> Result<Vec<CosmosMsg>, RelayTxError> {
    let msgs = match controller.auth.ty() {
        AuthenticatorType::Webauthn => {
            let webauthn_tx_msg: WebauthnRelayedTxMsg = from_binary(&transaction.message)?;

            let msg = authenticator_trait::sv::QueryMsg::Authenticate {
                // this is the JSON string of the VectisRelayTx, containing the Vec<CosmosMsg>
                signed_data: webauthn_tx_msg.signed_data.as_bytes().to_vec(),
                controller_data: controller.data.0,
                metadata: vec![webauthn_tx_msg.auth_data.0, webauthn_tx_msg.client_data.0],
                signature: transaction.signature.0,
            };

            if !deps
                .querier
                .query_wasm_smart(auth_addr.ok_or(RelayTxError::AuthenticatorNotFound)?, &msg)?
            {
                return Err(RelayTxError::SignatureVerificationError);
            }

            let vectis_relayed_tx: VectisRelayedTx =
                serde_json_wasm::from_str(&webauthn_tx_msg.signed_data)
                    .map_err(|_| RelayTxError::SerdeVectisRelayedTx)?;
            check_cosmos_relayed_tx(&vectis_relayed_tx, controller.nonce)?;
            vectis_relayed_tx.messages
        }
        _ => {
            return Err(RelayTxError::AuthenticatorNotSupported);
        }
    };
    Ok(msgs)
}
