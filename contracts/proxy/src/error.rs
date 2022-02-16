use cosmwasm_std::StdError;
use sc_wallet::RelayTxError;
use thiserror::Error;

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
    #[error("IsNotMultisig")]
    IsNotMultisig {},
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
    #[error("RelayTxError")]
    RelayTxError(RelayTxError),
}
