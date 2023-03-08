#![allow(clippy::derive_partial_eq_without_eq)]
use cosmwasm_std::{Addr, StdError, Uint128};
use cw_utils::ParseReplyError;
use thiserror::Error;

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum DaoItemsQueryError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Item not set for {0}")]
    ItemNotSet(String),
    #[error("Dao Addr Not Found")]
    DaoAddrNotFound,
}

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum RelayTxError {
    #[error("MismatchControllerAddr")]
    IsNotController {},
    #[error("NoncesAreNotEqual")]
    NoncesAreNotEqual {},
    #[error("SignatureVerificationError")]
    SignatureVerificationError {},
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
pub enum IbcError {
    #[error("Only supports unordered channels")]
    InvalidChannelOrder,
    #[error("Counterparty version must be '{0}'")]
    InvalidChannelVersion(&'static str),
    #[error("Connection id must be = '{0}'")]
    InvalidConnectionId(String),
    #[error("Port id must be = '{0}'")]
    InvalidPortId(String),
    #[error("Invalid source endpoint")]
    InvalidSrc,
    #[error("Invalid PacketMsg")]
    InvalidPacketMsg,
    #[error("Invalid PacketMsg.msg")]
    InvalidInnerMsg,
    #[error("Invalid Job id")]
    UnsupportedJobId,
    #[error("Invalid DAO action id")]
    InvalidDaoActionId,
}

#[derive(Error, Debug, PartialEq)]
pub enum FactoryError {
    #[error("{0}")]
    Std(#[from] StdError),
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
    InvalidNativeFund(Uint128, Uint128),
    #[error("GovecNotSet")]
    GovecNotSet {},
    #[error("Proxy cannot be instantiated")]
    ProxyInstantiationError {},
    #[error("ClaimExpired")]
    ClaimExpired {},
    #[error("Invalid Govec Minter")]
    InvalidGovecMinter {},
    #[error("Invalid Govec Reply")]
    InvalidReplyFromGovec {},
    #[error("ParseReplyError")]
    ParseReplyError(#[from] ParseReplyError),
    #[error("Feature Not Supported By Current Chain")]
    NotSupportedByChain {},
    #[error("Claim fee required")]
    ClaimFeeRequired,
}

impl From<MigrationMsgError> for FactoryError {
    fn from(error: MigrationMsgError) -> Self {
        FactoryError::InvalidMigrationMsg(error)
    }
}
