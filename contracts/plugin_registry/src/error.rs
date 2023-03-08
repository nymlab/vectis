use cosmwasm_std::{StdError, Uint128};
use cw_utils::ParseReplyError;
use thiserror::Error;
use vectis_wallet::DaoItemsQueryError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DaoItemsQueryError(#[from] DaoItemsQueryError),

    #[error("{0}")]
    ParseReply(#[from] ParseReplyError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Plugin Approval Committee Not Found")]
    PluginCommitteeNotFound,

    #[error("Checksum Verification Failed")]
    ChecksumVerificationFailed,

    #[error("Not Supported Reply Id")]
    NotSupportedReplyId,

    #[error("Insufficient Fee Amount: Expected: {0}, Got: {1}")]
    InsufficientFee(Uint128, Uint128),
}
