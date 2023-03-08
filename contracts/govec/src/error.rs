use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;
use vectis_wallet::DaoItemsQueryError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DaoItemsQueryError(#[from] DaoItemsQueryError),

    #[error("{0}")]
    Cw20Stake(#[from] cw20_stake::ContractError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},

    #[error("Incorrect voting power for burning: {0}")]
    IncorrectBalance(Uint128),

    #[error("Wallet not found")]
    NotFound {},
}
