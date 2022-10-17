use cosmwasm_std::{Addr, Coin};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct UnclaimedWalletList {
    pub wallets: Vec<(Addr, Expiration)>,
}
