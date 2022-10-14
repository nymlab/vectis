use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use vectis_wallet::{
    CodeIdType, CreateWalletMsg, ProxyMigrationTxMsg, StakingOptions, WalletAddr,
    WalletFactoryExecuteMsg as ExecuteMsg, WalletFactoryInstantiateMsg as InstantiateMsg,
    WalletFactoryQueryMsg as QueryMsg,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct WalletListResponse {
    pub wallets: Vec<Addr>,
}
