use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

pub const ITEMS: Map<String, String> = Map::new("items");
pub const DAO: Item<CanonicalAddr> = Item::new("dao_addr");
