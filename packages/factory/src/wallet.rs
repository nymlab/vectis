use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Binary, CanonicalAddr};

/// User nonce
pub type Nonce = u64;

/// Representation of the wallet address in both form used in migration
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum WalletAddr {
    /// CanonicalAddr
    Canonical(CanonicalAddr),
    /// Addr
    Addr(Addr),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct WalletInfo {
    pub user_addr: Addr,
    pub version: cw2::ContractVersion,
    pub code_id: u64,
    pub guardians: Vec<Addr>,
    pub relayers: Vec<Addr>,
    pub is_frozen: bool,
    pub nonce: Nonce,
    pub multisig_address: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RelayTransaction {
    /// User pubkey
    pub user_pubkey: Binary,
    /// Message to verify
    pub message: Binary,
    /// Nonce
    pub nonce: Nonce,
    /// Serialized signature (message + nonce). Cosmos format (64 bytes).
    /// Cosmos format (secp256k1 verification scheme).
    pub signature: Binary,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum ProxyMigrationMsg {
    RelayTx(RelayTransaction),
    DirectMigrationMsg(Binary),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MigrateMsg {
    pub new_code_id: u64,
}
