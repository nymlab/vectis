pub mod contract;
mod error;
mod execute;
pub mod helpers;
pub mod msg;
mod query;
pub mod state;

pub use crate::error::ContractError;

#[cfg(test)]
mod tests;

use cw_utils::Duration;

/// Max voting is set to > 7 years
pub(crate) const MAX_MULTISIG_VOTING_PERIOD: Duration = Duration::Time(2 << 27);
// set resasonobly high value and not interfere with multisigs
/// Used to spot an multisig instantiate reply
pub(crate) const MULTISIG_INSTANTIATE_ID: u64 = u64::MAX;
pub(crate) const MULTISIG_ROTATION_ID: u64 = u64::MAX - 1u64;
pub(crate) const PLUGIN_INST_ID: u64 = u64::MAX - 2u64;
pub(crate) const REG_PLUGIN_INST_ID: u64 = u64::MAX - 3u64;
