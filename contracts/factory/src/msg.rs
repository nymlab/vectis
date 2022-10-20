use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_utils::Expiration;
pub use vectis_wallet::{
    UnclaimedWalletList, WalletFactoryExecuteMsg as ExecuteMsg,
    WalletFactoryInstantiateMsg as InstantiateMsg, WalletFactoryQueryMsg as QueryMsg,
};
