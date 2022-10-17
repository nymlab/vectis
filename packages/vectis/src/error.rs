use thiserror::Error;

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum RelayTxError {
    #[error("MismatchUserAddr")]
    IsNotUser {},
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
}
