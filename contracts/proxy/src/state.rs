use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};
use vectis_wallet::{Nonce, RelayTxError};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct User {
    pub addr: CanonicalAddr,
    pub nonce: Nonce,
}

impl User {
    /// Increase nonce by 1
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Set new user address
    pub fn set_address(&mut self, address: CanonicalAddr) {
        self.addr = address;
    }

    /// Ensure nonces are equal
    pub fn ensure_nonces_are_equal(&self, nonce: &Nonce) -> Result<(), ContractError> {
        if self.nonce.eq(nonce) {
            Ok(())
        } else {
            Err(ContractError::RelayTxError(
                RelayTxError::NoncesAreNotEqual {},
            ))
        }
    }

    /// Ensure provided address is different from current.
    pub fn ensure_addresses_are_not_equal(
        &self,
        address: &CanonicalAddr,
    ) -> Result<(), ContractError> {
        if self.addr.ne(address) {
            Ok(())
        } else {
            Err(ContractError::AddressesAreEqual {})
        }
    }
}

pub const FROZEN: Item<bool> = Item::new("frozen");
pub const FACTORY: Item<CanonicalAddr> = Item::new("factory");
pub const CODE_ID: Item<u64> = Item::new("code_id");
pub const MULTISIG_CODE_ID: Item<u64> = Item::new("multisig_code_id");
pub const USER: Item<User> = Item::new("user");
pub const GUARDIANS: Map<&[u8], ()> = Map::new("guardians");
pub const RELAYERS: Map<&[u8], ()> = Map::new("relayers");
// An address of fixed multisig contract, used for guardians multisig support.
pub const MULTISIG_ADDRESS: Item<Option<CanonicalAddr>> = Item::new("fixed_multisig_address");
/// Chain address prefix
pub const ADDR_PREFIX: Item<String> = Item::new("addr_prefix");
