use cosmwasm_std::Addr;
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use vectis_wallet::{
    WalletFactoryExecuteMsg as ExecuteMsg, WalletFactoryInstantiateMsg as InstantiateMsg,
    WalletFactoryQueryMsg as QueryMsg,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct UnclaimedWalletList {
    pub wallets: Vec<(Addr, Expiration)>,
}
