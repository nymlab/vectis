use crate::wallet::{ProxyMigrationTxMsg, WalletAddr};
use cosmwasm_std::{Binary, Coin};
use cw20::Cw20Coin;
use govec::msg::StakingOptions;
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Default)]
pub struct MultiSig {
    // Declares that a fixed weight of Yes votes is needed to pass.
    /// Only Fixed multisig is supported in this version
    pub threshold_absolute_count: ThresholdAbsoluteCount,
    // intial funds for multisig contract
    pub multisig_initial_funds: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WalletFactoryQueryMsg {
    /// Shows proxy wallet address
    /// Returns WalletListResponse
    Wallets {
        // Address string to start after
        start_after: Option<(String, String)>,
        // Max is 30 and default is 10
        limit: Option<u32>,
    },
    WalletsOf {
        user: String,
        // Address string to start after
        start_after: Option<String>,
        // Max is 30 and default is 10
        limit: Option<u32>,
    },
    CodeId {
        ty: CodeIdType,
    },
    /// Returns the fee required to create a wallet
    /// Fee goes to the DAO
    Fee {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WalletFactoryExecuteMsg {
    CreateWallet {
        create_wallet_msg: CreateWalletMsg,
    },
    MigrateWallet {
        wallet_address: WalletAddr,
        migration_msg: ProxyMigrationTxMsg,
    },
    UpdateCodeId {
        ty: CodeIdType,
        new_code_id: u64,
    },
    UpdateWalletFee {
        new_fee: Coin,
    },
    CreateGovernance {
        staking_options: Option<StakingOptions>,
        initial_balances: Vec<Cw20Coin>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum CodeIdType {
    Proxy,
    Multisig,
    Govec,
    Staking,
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
