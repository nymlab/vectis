use cw_storage_plus::{Item, Map};
use vectis_wallet::DaoConfig;

/// The config for communicating with the DAO
pub const DAO_CONFIG: Item<DaoConfig> = Item::new("dao_config");

/// We store approved IBC transfer module connections:
/// local connection_id: the light client of the remote chain
/// we don't track caller port id as it is bounded to the ibctransfer module on the remote chain
///
/// This just tracks existing channels, mainly to be used by the dao
/// to send funds from this contract out
pub const IBC_TRANSFER_MODULES: Map<String, String> = Map::new("ibc_transfer_modules");

/// Job Id for current dispatch,
/// used for listening for callbacks for users.
/// Will loop around if max is hit
pub const JOB_ID: Item<u64> = Item::new("job_id");

// We also use ITEMS to store DaoActors::Factory
