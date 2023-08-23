use crate::types::{plugin::Plugin, wallet::Controller};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CanonicalAddr};
use cw_storage_plus::{Item, Map};

// Stored on the Multisig / DAO contract
/// Maps `VectisActors` variant to addresses:
/// using `String` as key to keep inline with the dao-core contract
pub const ITEMS: Map<String, String> = Map::new("items");
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

// Stores the controller of the wallet
pub type ControllerState<'a> = Item<'a, Controller>;
pub const CONTROLLER: ControllerState = Item::new("controller");

// Stored on all Vectis contract help find other VectisActors
/// In v1.0.0 this is a multisig
/// In progressive decentralisation this will be a DAAO
pub type Deployer<'a> = Item<'a, CanonicalAddr>;
pub const DEPLOYER: Deployer = Item::new("deployer");

// Stored on Factory
/// These are the authenticators types stored on the factory
pub type Authenticators<'a> = Map<'a, String, Addr>;
pub const AUTHENTICATORS: Authenticators = Map::new("authenticators");

// Stored on Plugin registry
/// Types that allow other contracts to query the plugins from the registry
pub type Plugins<'a> = Map<'a, u64, Plugin>;
pub const PLUGINS: Plugins = Map::new("registry_plugins");
