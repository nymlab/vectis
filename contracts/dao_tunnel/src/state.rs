use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

/// We store approved connection and port id, whilst allowing multiple channels to be created
pub const IBC_CONTROLLERS: Map<(String, String), ()> = Map::new("ibc_controllers");

/// The admin where the fees for new wallet goes to, also the admin of the contract.
/// Likely a DAO
pub const ADMIN: Item<CanonicalAddr> = Item::new("admin");
