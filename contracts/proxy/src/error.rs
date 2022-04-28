use cosmwasm_std::StdError;
use thiserror::Error;
use vectis_wallet::RelayTxError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Frozen")]
    Frozen {},
    #[error("InvalidMessage")]
    InvalidMessage {},
    #[error("IsNotGuardian")]
    IsNotGuardian {},
    #[error("IsNotRelayer")]
    IsNotRelayer {},
    #[error("IsNotContractSelf: Relayed message should be from target contract")]
    IsNotContractSelf {},
    #[error("IsNotMultisig")]
    IsNotMultisig {},
    #[error("IsNotUser")]
    IsNotUser {},
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
}
