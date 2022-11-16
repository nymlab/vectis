use cosmwasm_std::{Binary, DepsMut, StdResult};
use ripemd160::Digest as Ripemd160Digest;
use sha2::Sha256;

use crate::RelayTransaction;

pub fn query_verify_cosmos(deps: &DepsMut, transaction: &RelayTransaction) -> StdResult<bool> {
    let message_with_nonce = Binary(
        transaction
            .message
            .0
            .iter()
            .chain(&transaction.nonce.to_be_bytes())
            .copied()
            .collect(),
    );
    let hash = Sha256::digest(&message_with_nonce);
    let result = deps.api.secp256k1_verify(
        hash.as_ref(),
        &transaction.signature,
        &transaction.user_pubkey,
    )?;
    Ok(result)
}
