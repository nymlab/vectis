use cosmwasm_std::StdError;
use thiserror::Error;
use vectis_wallet::RelayTxError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Frozen")]
    Frozen {},
    #[error("InvalidMessage: {msg}")]
    InvalidMessage { msg: String },
    #[error("IsNotGuardian")]
    IsNotGuardian {},
    #[error("IsNotRelayer")]
    IsNotRelayer {},
    #[error("IsNotContractSelf: Relayed message should be from target contract")]
    IsNotContractSelf {},
    #[error("IsNotMultisig")]
    IsNotMultisig {},
    #[error("IsNotController")]
    IsNotController {},
    #[error("PubKeyIsNotValid")]
    PubKeyIsNotValid {},
    #[error("PubKeyLengthIsNotValid")]
    PubKeyLengthIsNotValid {},
    #[error("SameCodeId")]
    SameCodeId {},
    #[error("AddressesAreEqual")]
    AddressesAreEqual {},
    #[error("RelayerDoesNotExist")]
    RelayerDoesNotExist {},
    #[error("RelayerAlreadyExists")]
    RelayerAlreadyExists {},
    #[error("IncorrectThreshold")]
    IncorrectThreshold {},
    #[error("RelayTxError")]
    RelayTxError(RelayTxError),
    #[error("InvalidReplyId")]
    InvalidReplyId {},
    #[error("Multisig cannot be instantiated")]
    MultisigInstantiationError {},
    #[error("Plugin cannot be instantiated")]
    PluginInstantiationError {},
    #[error("Same Label")]
    SameLabel {},
    #[error("No guardian request found")]
    GuardianRequestNotFound {},
    #[error("Guardian update request cannot be executed yet")]
    GuardianRequestNotExecutable {},
    #[error("Feature is not yet supported")]
    FeatureNotSupported,
    #[error("DAO Actor Contract Not Found")]
    ContractNotFound,
}
