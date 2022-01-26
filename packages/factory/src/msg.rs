use crate::wallet::{ProxyMigrationMsg, WalletAddr};
use cosmwasm_std::{Binary, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Declares that a fixed weight of Yes votes is needed to pass.
/// See `ThresholdResponse.AbsoluteCount` in the cw3 spec for details.
/// Only Fixed multisig is supported in this version
pub type ThresholdAbsoluteCount = u64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CreateWalletMsg {
    pub user_pubkey: Binary,
    pub guardians: Guardians,
    /// A List of keys can act as relayer for
    pub relayers: Vec<String>,
    pub proxy_initial_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Guardians {
    /// A List of keys can act as guardian for
    pub addresses: Vec<String>,
    /// Whether multisig option for guardians is enabled
    pub guardians_multisig: Option<MultiSig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MultiSig {
    // Declares that a fixed weight of Yes votes is needed to pass.
    /// Only Fixed multisig is supported in this version
    pub threshold_absolute_count: ThresholdAbsoluteCount,
    // intial funds for multisig contract
    pub multisig_initial_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WalletFactoryExecuteMsg {
    // do something here!
    CreateWallet {
        create_wallet_msg: CreateWalletMsg,
    },
    MigrateWallet {
        wallet_address: WalletAddr,
        proxy_migration_msg: ProxyMigrationMsg,
    },
    UpdateProxyCodeId {
        new_code_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct WalletInit {
    /// User pubkey
    pub user_pubkey: Binary,
    /// Message to verify
    pub message: Binary,
    /// Serialized signature. Cosmos format (64 bytes).
    /// Cosmos format (secp256k1 verification scheme).
    pub signature: Binary,
}
