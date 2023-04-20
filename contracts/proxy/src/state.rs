use cosmwasm_schema::cw_serde;
use vectis_wallet::PluginPermissions;

use crate::error::ContractError;
use cosmwasm_std::{Addr, CanonicalAddr, Coin};
use cw_storage_plus::{Item, Map};
pub use vectis_wallet::{
    factory_state::{ADDR_PREFIX, DEPLOYER, PROXY_MULTISIG_CODE_ID},
    GuardiansUpdateRequest, Nonce, RelayTxError, QUERY_PLUGINS,
};

#[cw_serde]
pub struct Controller {
    pub addr: CanonicalAddr,
    pub nonce: Nonce,
}

impl Controller {
    /// Increase nonce by 1
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Set new controller address
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
pub const CODE_ID: Item<u64> = Item::new("code_id");
pub const CONTROLLER: Item<Controller> = Item::new("controller");
pub const GUARDIANS: Map<&[u8], ()> = Map::new("guardians");
pub const PENDING_GUARDIAN_ROTATION: Item<GuardiansUpdateRequest> =
    Item::new("pending_guardian_rotation");
pub const RELAYERS: Map<&[u8], ()> = Map::new("relayers");
pub const LABEL: Item<String> = Item::new("label");
// An address of fixed multisig contract, used for guardians multisig support.
pub const MULTISIG_ADDRESS: Item<Option<CanonicalAddr>> = Item::new("fixed_multisig_address");
pub const INSTALL_FEE: Item<Coin> = Item::new("install_fee");
/// Plugins with execution rights
pub const EXEC_PLUGINS: Map<&[u8], ()> = Map::new("exec-plugins");
/// Plugins that do tx Pre-checks
pub const PRE_TX_PLUGINS: Map<&[u8], ()> = Map::new("pre-tx-plugins");
/// Plugins that is multisig  
pub const MULTISIG_PLUGIN: Item<CanonicalAddr> = Item::new("multisig-plugin");

// Tmp storage
// (controller, guardians)
pub const PENDING_MULTISIG: Item<(Addr, Vec<Addr>)> = Item::new("pending-multisig");
pub const PENDING_PLUGIN: Item<Vec<PluginPermissions>> = Item::new("pending-plugin");
