use cosmwasm_std::{Addr, DepsMut, StdError, StdResult};

use bech32::{ToBase32, Variant};
use primitive_types::H256;
use ripemd160::Digest as Ripemd160Digest;
use ripemd160::Ripemd160;
use sha2::Sha256;

/// Converts user pubkey into Addr
pub fn pub_key_to_address(deps: &DepsMut, pub_key: &[u8]) -> StdResult<Addr> {
    let compressed_pub_key = to_compressed_pub_key(pub_key)?;
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(Sha256::digest(&compressed_pub_key));
    let addr_bytes = ripemd160_hasher.finalize().to_vec();
    let addr_str = bech32::encode("wasm", addr_bytes.to_base32(), Variant::Bech32).unwrap();
    deps.api.addr_validate(&addr_str)
}

/// Converts uncompressed pub key into compressed one
fn to_compressed_pub_key(pub_key: &[u8]) -> StdResult<Vec<u8>> {
    match pub_key.len() {
        // compressed
        33 => Ok(pub_key.to_vec()),
        // uncompressed
        65 => {
            let y = H256::from_slice(&pub_key[33..]);
            let mut pub_key_compressed = pub_key[1..33].to_vec();

            // Check whether even or odd
            if y & H256::from_low_u64_be(1) == H256::zero() {
                // 0x02
                pub_key_compressed.insert(0, 2);
            } else {
                // 0x03
                pub_key_compressed.insert(0, 3);
            }

            Ok(pub_key_compressed)
        }
        _ => Err(StdError::GenericErr {
            msg: "PubKeyLengthIsNotValid".into(),
        }),
    }
}
