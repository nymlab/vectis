use cosmwasm_schema::cw_serde;
use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

/// Maps `VectisActors` variant to addresses:
/// using `String` as key to keep inline with the dao-core contract
pub const ITEMS: Map<String, String> = Map::new("items");
/// In beta-V1 this is a multisig
/// In progressive decentralisation this will be a DAAO
pub const DEPLOYER: Item<CanonicalAddr> = Item::new("deployer");

#[cw_serde]
pub enum VectisActors {
    Factory,
    PluginCommittee,
    PluginRegistry,
}

impl std::fmt::Display for VectisActors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Vectis Proxy state for other contract to query it
pub const QUERY_PLUGINS: Map<&str, CanonicalAddr> = Map::new("query-plugins");
