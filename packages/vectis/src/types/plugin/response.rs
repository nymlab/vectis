use cosmwasm_schema::cw_serde;

use super::types::*;

#[cw_serde]
pub struct PluginListResponse {
    pub ty: PluginPermission,
    pub plugins: Vec<(String, PluginInfo)>,
}

#[cw_serde]
pub struct PluginsResponse {
    pub plugins: Vec<Plugin>,
    pub total: u64,
    pub current_plugin_id: u64,
}

#[cw_serde]
pub struct PluginWithVersionResponse {
    pub contract_version: String,
    pub plugin_info: Plugin,
}
