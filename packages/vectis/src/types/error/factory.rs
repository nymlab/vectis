use super::*;

/// Contract migration related errors
#[derive(Error, Debug, PartialEq)]
pub enum MigrationMsgError {
    #[error("InvalidWalletAddr")]
    InvalidWalletAddr,
    #[error("MismatchProxyCodeId")]
    MismatchProxyCodeId,
    #[error("InvalidWasmMsg")]
    InvalidWasmMsg,
    #[error("MultisigFeatureIsNotSet")]
    MultisigFeatureIsNotSet,
    #[error("IsNotAProxyMsg")]
    IsNotAProxyMsg,
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
    Unauthorized,
    #[error("Missing Duration")]
    MissingDuration {},
    #[error("InvalidMigrationMsg: {0}")]
    InvalidMigrationMsg(MigrationMsgError),
    #[error("InvalidRelayMigrationTx: {0}")]
    InvalidRelayMigrationTx(RelayTxError),
    #[error("InvalidNativeFund: Expected: {0}{1}")]
    InvalidSufficientFunds(String, String),
    #[error("Proxy cannot be instantiated")]
    ProxyInstantiationError {},
    #[error("Invalid Proxy Reply")]
    InvalidReplyFromProxy,
}

impl From<MigrationMsgError> for FactoryError {
    fn from(error: MigrationMsgError) -> Self {
        FactoryError::InvalidMigrationMsg(error)
    }
}
