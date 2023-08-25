use super::*;

/// Contract migration related errors
#[derive(Error, Debug, PartialEq)]
pub enum MigrationMsgError {
    #[error("InvalidWalletAddr")]
    InvalidWalletAddr,
    #[error("MismatchProxyCodeId")]
    MismatchProxyCodeId,
    #[error("InvalidMigrationMsg")]
    InvalidMigrationMsg,
}

#[derive(Error, Debug, PartialEq)]
pub enum FactoryError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Instantiate2(#[from] Inst2CalcError),
    #[error("RelayTxError {0}")]
    RelayTxError(#[from] RelayTxError),
    #[error("NotFound")]
    NotFound { addr: Addr },
    #[error("OverFlow")]
    OverFlow {},
    #[error("SameProxyCodeId")]
    SameProxyCodeId {},
    #[error("Unauthorized")]
    Unauthorized,
    #[error("InvalidMigrationMsg: {0}")]
    InvalidMigrationMsg(MigrationMsgError),
    #[error("InvalidRelayMigrationTx: {0}")]
    InvalidRelayMigrationTx(RelayTxError),
    #[error("InvalidNativeFund: Expected: {0}{1}")]
    InvalidSufficientFunds(String, String),
    #[error("Proxy cannot be instantiated")]
    ProxyInstantiationError {},
    #[error("Unexpected Update Params")]
    UnexpectedUpdateParams,
    #[error("AlreadyExist {addr}")]
    AlreadyExist { addr: Addr },
}

impl From<MigrationMsgError> for FactoryError {
    fn from(error: MigrationMsgError) -> Self {
        FactoryError::InvalidMigrationMsg(error)
    }
}
