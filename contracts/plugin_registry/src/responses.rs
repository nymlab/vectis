use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;

use crate::contract::Plugin;

#[cw_serde]
pub struct PluginsResponse {
    pub plugins: Vec<Plugin>,
    pub total: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub registry_fee: Coin,
    pub deployer_addr: String,
}
