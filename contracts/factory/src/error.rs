use cosmwasm_std::{Addr, StdError, Uint128};
use sc_wallet::{MigrationMsgError, RelayTxError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("EmptyGuardians")]
    EmptyGuardians {},
    #[error("ThresholdShouldBeGreaterThenZero")]
    ThresholdShouldBeGreaterThenZero {},
    #[error("ThresholdShouldBeLessThenGuardiansCount")]
    ThresholdShouldBeLessThenGuardiansCount {},
    #[error("AlreadyExist")]
    AlreadyExist { addr: Addr },
    #[error("NotFound")]
    NotFound { addr: Addr },
    #[error("OverFlow")]
    OverFlow {},
    #[error("SameProxyCodeId")]
    SameProxyCodeId {},
    #[error("SameProxyMultisigCodeId")]
    SameProxyMultisigCodeId {},
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("InvalidMigrationMsg: {0}")]
    InvalidMigrationMsg(MigrationMsgError),
    #[error("InvalidRelayMigrationTx: {0}")]
    InvalidRelayMigrationTx(RelayTxError),
    #[error("InvalidReplyId")]
    InvalidReplyId {},
    #[error("InvalidNativeFund: Expected: {1}, Got: {0}")]
    InvalidNativeFund(Uint128, Uint128),
}

impl From<MigrationMsgError> for ContractError {
    fn from(error: MigrationMsgError) -> Self {
        ContractError::InvalidMigrationMsg(error)
    }
}
