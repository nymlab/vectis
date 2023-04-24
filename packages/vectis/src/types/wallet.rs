use crate::{ProxyAddrErr, RelayTxError};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CanonicalAddr};
use cw_utils::Threshold;

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
    pub fn ensure_nonces_are_equal(&self, nonce: &Nonce) -> Result<(), RelayTxError> {
        if self.nonce.eq(nonce) {
            Ok(())
        } else {
            Err(RelayTxError::NoncesAreNotEqual {})
        }
    }

    /// Ensure provided address is different from current.
    pub fn ensure_addresses_are_not_equal(
        &self,
        address: &CanonicalAddr,
    ) -> Result<(), ProxyAddrErr> {
        if self.addr.ne(address) {
            Ok(())
        } else {
            Err(ProxyAddrErr::AddressesAreEqual {})
        }
    }
}

/// Controller nonce
pub type Nonce = u64;

/// Representation of the wallet address in both form used in migration
#[cw_serde]
pub enum WalletAddr {
    /// CanonicalAddr
    Canonical(CanonicalAddr),
    /// Addr
    Addr(Addr),
}

#[cw_serde]
pub struct WalletInfo {
    pub controller_addr: Addr,
    pub deployer: Addr,
    pub version: cw2::ContractVersion,
    pub code_id: u64,
    pub guardians: Vec<Addr>,
    pub relayers: Vec<Addr>,
    pub is_frozen: bool,
    pub created_at: u64,
    pub nonce: Nonce,
    pub multisig_address: Option<Addr>,
    pub multisig_threshold: Option<Threshold>,
    pub label: String,
}

#[cw_serde]
pub struct RelayTransaction {
    /// Controller pubkey
    pub controller_pubkey: Binary,
    /// Message to verify
    pub message: Binary,
    /// Nonce
    pub nonce: Nonce,
    /// Serialized signature (message + nonce). Cosmos format (64 bytes).
    /// Cosmos format (secp256k1 verification scheme).
    pub signature: Binary,
}

#[cw_serde]
pub struct PluginListResponse {
    pub exec_plugins: Vec<Addr>,
    pub query_plugins: Vec<Addr>,
    pub pre_tx_plugins: Vec<Addr>,
    pub multisig_override: Option<Addr>,
}

#[cw_serde]
pub struct WalletCreateReply {
    pub controller: Addr,
    pub proxy_addr: Addr,
    pub multisig_addr: Option<Addr>,
    pub guardians: Vec<Addr>,
}

/// Permission of the plugin on the proxy
#[cw_serde]
pub enum PluginPermissions {
    /// Can Exec through Proxy
    Exec,
    /// Addr can be queried through proxy
    Query(String),
    /// Is used to check tx before execution
    PreTxCheck,
    /// Is a multisig contract, ignore controller
    MultiSigOverride,
}
