pub mod contract;
mod error;
pub mod msg;
pub mod state;
pub mod ibc;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;

pub const FACTORY_CALLBACK_ID: u64 = 7890;