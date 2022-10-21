use cosmwasm_std::{Binary, CanonicalAddr};
use cw_storage_plus::{Item, Map};

/// We store approved:
/// local connection_id: the light client of the remote chain
/// caller port id: bounded to the wasm contract addr on the remote chain
///
/// This allows for multiple channels to be created between dao and remote tunnels
pub const IBC_TUNNELS: Map<(String, String), ()> = Map::new("ibc_tunnels");

/// The admin where the fees for new wallet goes to, also the admin of the contract.
/// Likely a DAO
pub const ADMIN: Item<CanonicalAddr> = Item::new("admin");

/// Govec Addr
pub const GOVEC: Item<CanonicalAddr> = Item::new("govec");

/// Stores all results from current dispatch
pub const RESULTS: Item<Vec<Binary>> = Item::new("results");
