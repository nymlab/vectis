pub mod contract;
mod error;
pub mod ibc;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

pub use crate::error::ContractError;

pub const FACTORY_CALLBACK_ID: u64 = 7890;
pub const DISPATCH_CALLBACK_ID: u64 = 7891;
