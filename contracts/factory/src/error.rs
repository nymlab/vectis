use cosmwasm_std::{Addr, StdError};
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
    InvalidRelayMigrationTx(RelayMigrationError),
}

#[derive(Error, Debug, PartialEq)]
pub enum MigrationMsgError {
    #[error("InvalidWalletAddr")]
    InvalidWalletAddr,
    #[error("MismatchCodeId")]
    MismatchCodeId,
    #[error("InvalidWasmMsg")]
    InvalidWasmMsg,
}

#[derive(Error, Debug, PartialEq)]
pub enum RelayMigrationError {
    #[error("MismatchUserAddr")]
    MismatchUserAddr,
    #[error("MismatchNonce")]
    MismatchNonce,
    #[error("SignatureVerificationError")]
    SignatureVerificationError,
}
