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
