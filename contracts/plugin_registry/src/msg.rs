use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CanonicalAddr, Coin};

// We can store information offchain like number of installs per plugin, review of plugins ?
#[cw_serde]
pub struct Plugin {
    pub id: u64,
    pub name: String,
    pub creator: CanonicalAddr,
    // (Optional)
    pub ipfs_hash: String,
    // We enforce using semver for versioning
    pub version: String,
    // Just one chain because could be different version between chains.
    pub chain_id: String,
    pub code_id: u64,
    pub checksum: String,
    // None means free
    // shall we support different denoms than native?
    pub price: Option<Coin>,
    // permissions required by the plugin.
    pub permissions: Vec<String>,
}

#[cw_serde]
pub struct RegistryConfig {
    pub revieweres: Vec<String>,
    pub supported_denoms: Vec<String>,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub reviewers: Vec<String>,
    pub supported_denoms: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterPlugin {
        name: String,
        version: String,
        ipfs_hash: String,
        chain_id: String,
        code_id: u64,
        checksum: String,
        permissions: Vec<String>,
    },
    UnregisterPlugin {
        id: u64,
    },
    UpdatePlugin {
        id: u64,
        name: Option<String>,
        version: String,
        ipfs_hash: Option<String>,
        code_id: Option<u64>,
        checksum: Option<String>,
        permissions: Option<Vec<String>>,
    },
    BuyPlugin {
        id: u64,
    },
    UpdateConfig {
        reviewers: Option<Vec<String>>,
        supported_denoms: Option<Vec<String>>,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetPlugins {
        limit: Option<u32>,
        start_after: Option<u32>,
    },
    GetPluginsByAddr {
        addr: String,
        limit: Option<u32>,
        start_after: Option<u32>,
    },
    GetPluginById {
        id: u64,
    },
}
