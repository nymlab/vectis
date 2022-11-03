use cw_storage_plus::{Item, Map};
use vectis_wallet::{ChainConfig, DaoConfig};

/// The config for communicating with the DAO
pub const DAO_CONFIG: Item<DaoConfig> = Item::new("dao_config");
/// The config for this chain
pub const CHAIN_CONFIG: Item<ChainConfig> = Item::new("chain_config");

/// We store approved IBC transfer module connections:
/// local connection_id: the light client of the remote chain
/// caller port id: bounded to the wasm contract addr on the remote chain
///
/// This allows for multiple channels to be created between dao and remote tunnels
pub const IBC_TRANSFER_MODULES: Map<(String, String), Option<String>> =
    Map::new("ibc_transfer_modules");

/// Job Id for current dispatch,
/// used for listening for callbacks for users.
/// Will loop around if max is hit
pub const JOB_ID: Item<u64> = Item::new("job_id");
