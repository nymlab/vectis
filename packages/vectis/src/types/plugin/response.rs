use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cosmwasm_std::Coin;

use super::types::*;

#[cw_serde]
pub struct PluginListResponse {
    pub exec: Vec<(Addr, PluginInfo)>,
    pub pre_tx: Vec<(Addr, PluginInfo)>,
    pub post_tx_hooks: Vec<(Addr, PluginInfo)>,
}

#[cw_serde]
pub struct PluginsResponse {
    pub plugins: Vec<Plugin>,
    pub total: u64,
    pub current_plugin_id: u64,
}

#[cw_serde]
pub struct ConfigResponse {
    pub registry_fee: Coin,
    pub deployer_addr: String,
}

#[cw_serde]
pub struct Fees {
    pub install: Coin,
    pub registry: Coin,
}

#[cw_serde]
pub struct PluginWithVersionResponse {
    pub contract_version: String,
    pub plugin_info: Plugin,
}
