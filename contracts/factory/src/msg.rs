use cosmwasm_std::{Addr, Uint128};
pub use sc_wallet::{WalletFactoryExecuteMsg as ExecuteMsg, WalletFactoryQueryMsg as QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Smart contract wallet contract code id
    pub proxy_code_id: u64,
    /// Wallet guardians multisig contract code id
    /// Currently v0.13.0 cw-plus cw3-fixed-multisig
    pub proxy_multisig_code_id: u64,
    /// Chain address prefix
    pub addr_prefix: String,
    /// Native token denom
    pub coin_denom: String,
    /// Fee in native token to be sent to Admin (DAO)
    pub wallet_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct WalletListResponse {
    pub wallets: Vec<Addr>,
}
