use cosmwasm_std::StdError;
use thiserror::Error;
use vectis_wallet::IbcError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    IbcError(#[from] IbcError),

    #[error("Invalid controller")]
    InvalidController {},

    #[error("Invalid Dispatch")]
    InvalidDispatch {},

    #[error("Unauthorized")]
    Unauthorized,
}
