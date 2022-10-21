use thiserror::Error;

use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;

use vectis_wallet::IbcError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    IbcError(#[from] IbcError),
    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),
    #[error("Invalid reply id")]
    InvalidReplyId,
    #[error("{0} not found")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Empty Funds")]
    EmptyFund {},
}
