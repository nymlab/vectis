use crate::msg::Guardians;
use crate::MigrationMsgError;
use cosmwasm_std::{Addr, Binary, CanonicalAddr};
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
    // code if of multisig contract utilised by wallet instance
    pub multisig_code_id: u64,
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
    pub fn ensure_is_proxy_msg(&self) -> Result<(), MigrationMsgError> {
        if let MigrateMsg::Proxy(_) = self {
            Ok(())
        } else {
            Err(MigrationMsgError::IsNotAProxyMsg)
        }
    }

    /// Ensure provided msg is multisig msg
    pub fn ensure_is_multisig_msg(&self) -> Result<(), MigrationMsgError> {
        if let MigrateMsg::Multisig(_) = self {
            Ok(())
        } else {
            Err(MigrationMsgError::IsNotAMultisigMsg)
        }
    }

    /// Ensures code id of multisig contract is equal to current factory multisig code id,
    /// Ensures new multisig_code id does not equal to current one if no guardians changes required
    pub fn ensure_is_correct_multisig_code_id(
        &self,
        factory_multisig_code_id: u64,
        proxy_multisig_code_id: u64,
    ) -> Result<(), MigrationMsgError> {
        if let MigrateMsg::Multisig(migrate_multisig_contract_msg) = self {
            if factory_multisig_code_id != migrate_multisig_contract_msg.new_multisig_code_id {
                return Err(MigrationMsgError::MismatchMultisigCodeId);
            }
            if migrate_multisig_contract_msg.new_guardians.is_some()
                && migrate_multisig_contract_msg.new_multisig_code_id == proxy_multisig_code_id
            {
                return Err(MigrationMsgError::MismatchMultisigCodeId);
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ProxyMigrateMsg {
    pub new_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MultisigMigrateMsg {
    // New guardians settings
    pub new_guardians: Option<Guardians>,
    pub new_multisig_code_id: u64,
}
