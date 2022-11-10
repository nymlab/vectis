use cosmwasm_std::StdError;
use thiserror::Error;
use vectis_wallet::IbcError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    IbcError(#[from] IbcError),
    #[error("Invalid remote tunnel")]
    InvalidTunnel,
    #[error("Invalid Dispatch")]
    InvalidDispatch {},
    #[error("Invalid reply id")]
    InvalidReplyId,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Channel not found for connection_id: {0}")]
    ChannelNotFound(String),
    #[error("Empty Funds")]
    EmptyFund,
}
