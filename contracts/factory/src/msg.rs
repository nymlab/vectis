use cosmwasm_std::Addr;
pub use sc_wallet::{WalletFactoryExecuteMsg as ExecuteMsg, WalletFactoryQueryMsg as QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub proxy_code_id: u64,
    pub proxy_multisig_code_id: u64,
    pub addr_prefix: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct WalletListResponse {
    pub wallets: Vec<Addr>,
}
