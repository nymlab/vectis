use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Insufficient Fee Amount: Expected: {0}, Got: {1}")]
    InsufficientFee(Uint128, Uint128),

    #[error("Registry Fee Required")]
    RegistryFeeRequired,
}
