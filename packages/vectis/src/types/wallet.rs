use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CanonicalAddr};

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
    pub nonce: Nonce,
    pub multisig_address: Option<Addr>,
    pub multisig_threshold: Option<u64>,
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
