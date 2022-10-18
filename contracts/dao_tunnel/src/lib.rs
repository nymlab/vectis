pub mod contract;
pub mod error;
pub mod ibc;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

pub const MING_DISPATCH_ID: u64 = 4321;
