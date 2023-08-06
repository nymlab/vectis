#![allow(clippy::derive_partial_eq_without_eq)]
use cosmwasm_std::{Addr, Instantiate2AddressError, StdError};
use cw_utils::ParseReplyError;
use thiserror::Error;

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum DeployerItemsQueryError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Item not set for {0}")]
    ItemNotSet(String),
    #[error("Deployer Addr Not Found")]
    DeployerAddrNotFound,
}

/// Checks of controller addr
#[derive(Error, Debug, PartialEq)]
pub enum ProxyAddrErr {
    #[error("Address Not Equal")]
    AddressesAreEqual {},
}

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum RelayTxError {
    #[error("MismatchControllerAddr")]
    IsNotController,
    #[error("NoncesAreNotEqual")]
    NoncesAreNotEqual,
    #[error("EmptyMsgs")]
    EmptyMsg,
    #[error("SignatureVerificationError")]
    SignatureVerificationError,
}

/// Contract migration related errors
#[derive(Error, Debug, PartialEq)]
pub enum MigrationMsgError {
    #[error("InvalidWalletAddr")]
    InvalidWalletAddr,
    #[error("MismatchProxyCodeId")]
    MismatchProxyCodeId,
    #[error("MismatchMultisigCodeId")]
    MismatchMultisigCodeId,
    #[error("InvalidWasmMsg")]
    InvalidWasmMsg,
    #[error("MultisigFeatureIsNotSet")]
    MultisigFeatureIsNotSet,
    #[error("IsNotAProxyMsg")]
    IsNotAProxyMsg,
    #[error("IsNotAMultisigMsg")]
    IsNotAMultisigMsg,
}

#[derive(Error, Debug, PartialEq)]
pub enum Inst2CalcError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Instantiate2(#[from] Instantiate2AddressError),
}

#[derive(Error, Debug, PartialEq)]
pub enum FactoryError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Instantiate2(#[from] Inst2CalcError),
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
    #[error("Missing Duration")]
    MissingDuration {},
    #[error("InvalidMigrationMsg: {0}")]
    InvalidMigrationMsg(MigrationMsgError),
    #[error("InvalidRelayMigrationTx: {0}")]
    InvalidRelayMigrationTx(RelayTxError),
    #[error("InvalidReplyId")]
    InvalidReplyId {},
    #[error("InvalidNativeFund: Expected: {0}, Got: {1}")]
    InvalidNativeFund(String, String),
    #[error("Proxy cannot be instantiated")]
    ProxyInstantiationError {},
    #[error("Invalid Proxy Reply")]
    InvalidReplyFromProxy,
    #[error("ParseReplyError")]
    ParseReplyError(#[from] ParseReplyError),
}

impl From<MigrationMsgError> for FactoryError {
    fn from(error: MigrationMsgError) -> Self {
        FactoryError::InvalidMigrationMsg(error)
    }
}
