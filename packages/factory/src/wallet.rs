use crate::msg::Guardians;
use cosmwasm_std::{Addr, Binary, CanonicalAddr, StdError, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
pub enum MigrateMsg {
    Proxy(ProxyMigrateMsg),
    Multisig(MultisigMigrateMsg),
}

impl MigrateMsg {
    /// Ensure provided msg is proxy msg
    pub fn ensure_is_proxy_msg(&self) -> StdResult<()> {
        if let MigrateMsg::Proxy(_) = self {
            Ok(())
        } else {
            Err(StdError::GenericErr {
                msg: "IsNotAProxyMsg".into(),
            })
        }
    }

    /// Ensure provided msg is multisig msg
    pub fn ensure_is_multisig_msg(&self) -> StdResult<()> {
        if let MigrateMsg::Multisig(_) = self {
            Ok(())
        } else {
            Err(StdError::GenericErr {
                msg: "IsNotAMultisigMsg".into(),
            })
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProxyMigrateMsg {
    pub new_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MultisigMigrateMsg {
    // New guardians setting
    pub new_guardians: Guardians,
    pub new_multisig_code_id: u64,
}
