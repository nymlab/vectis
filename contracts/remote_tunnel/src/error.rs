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
    #[error("DAO channel not found")]
    DaoChannelNotFound,
    #[error("Channel not found for connection_id: {0} and port_id {1}")]
    ChannelNotFound(String, String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Factory contract not available")]
    FactoryNotAvailable,
    #[error("Empty Funds")]
    EmptyFund {},
}
