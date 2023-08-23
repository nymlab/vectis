use super::*;

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum RelayTxError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("MismatchControllerAddr")]
    IsNotController,
    #[error("NoncesAreNotEqual")]
    NoncesAreNotEqual,
    #[error("EmptyMsgs")]
    EmptyMsg,
    #[error("SignatureVerificationError")]
    SignatureVerificationError,
    #[error("AuthenticatorNotFound")]
    AuthenticatorNotFound,
    #[error("AuthenticatorNotSupported")]
    AuthenticatorNotSupported,
    #[error("De VectisRelayTx")]
    SerdeVectisRelayedTx,
}

#[derive(Error, Debug, PartialEq)]
pub enum WalletError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("RelayTxError {0}")]
    RelayTxError(#[from] RelayTxError),
    #[error("{0}")]
    DeployerItemsQueryError(#[from] DeployerItemsQueryError),
    #[error("{0}")]
    EncodeError(#[from] EncodeError),
    #[error("{0}")]
    Inst2CalcErr(#[from] Inst2CalcError),
    #[error("{0}")]
    WalletPluginError(#[from] WalletPluginError),
    #[error("{0}")]
    PluginRegError(#[from] PluginRegError),
    #[error("InvalidMessage: {msg}")]
    InvalidMessage { msg: String },
    #[error("IsNotContractSelf: Relayed message should be from target contract")]
    IsNotContractSelf {},
    #[error("IsNotController")]
    IsNotController {},
    #[error("PubKeyIsNotValid")]
    PubKeyIsNotValid {},
    #[error("PubKeyLengthIsNotValid")]
    PubKeyLengthIsNotValid {},
    #[error("Feature is not yet supported")]
    FeatureNotSupported,
    #[error("Error in parsing msgs to JSON")]
    ParseError,
    #[error("Invalid authenticator")]
    InvalidAuthenticator,
}

#[derive(Error, Debug, PartialEq)]
pub enum WalletPluginError {
    #[error("Plugin cannot be instantiated")]
    PluginInstantiationError {},
    #[error("Plugin permission empty")]
    PermissionEmpty,
    #[error("Same plugin already installed")]
    PluginInstalled,
}
