use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use vectis_wallet::{
    CodeIdType, CreateWalletMsg, ProxyMigrationTxMsg, StakingOptions, WalletAddr,
    WalletFactoryQueryMsg as QueryMsg,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Smart contract wallet contract code id
    pub proxy_code_id: u64,
    /// Wallet guardians multisig contract code id
    /// Currently v0.13.0 cw-plus cw3-fixed-multisig
    pub proxy_multisig_code_id: u64,
    /// staking contract code id
    pub staking_code_id: u64,
    /// govec token contract code id
    pub govec_code_id: u64,
    /// Chain address prefix
    pub addr_prefix: String,
    /// Fee in native token to be sent to Admin (DAO)
    pub wallet_fee: Coin,
    /// Governance Token, Govec, address
    pub govec: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
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
    UpdateGovecAddr {
        addr: String,
    },
    UpdateAdmin {
        addr: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct WalletListResponse {
    pub wallets: Vec<Addr>,
}
