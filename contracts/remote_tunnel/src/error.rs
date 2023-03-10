use thiserror::Error;

use cosmwasm_std::StdError;
use cw_utils::{ParseReplyError, PaymentError};

use vectis_wallet::{DaoItemsQueryError, IbcError};

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    DaoItemsQueryError(#[from] DaoItemsQueryError),
    #[error("{0}")]
    IbcError(#[from] IbcError),
    #[error("{0}")]
    PaymentError(#[from] PaymentError),
    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),
    #[error("Invalid reply id")]
    InvalidReplyId,
    #[error("DAO channel not found")]
    DaoChannelNotFound,
    #[error("Channel not found for connection_id: {0}")]
    ChannelNotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Factory contract not available")]
    FactoryNotAvailable,
    #[error("Empty Funds")]
    EmptyFund,
}
