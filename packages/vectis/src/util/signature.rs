use cosmwasm_std::{DepsMut, StdResult};
use ripemd160::Digest as Ripemd160Digest;
use sha2::Sha256;

use crate::types::wallet::RelayTransaction;

pub fn verify_cosmos_sign(
    deps: &DepsMut,
    transaction: &RelayTransaction,
    pubkey: &[u8],
) -> StdResult<bool> {
    let hash = Sha256::digest(&transaction.message);
    let result = deps
        .api
        .secp256k1_verify(hash.as_ref(), &transaction.signature, &pubkey)?;
    Ok(result)
}
