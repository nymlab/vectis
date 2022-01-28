use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("SignatureVerificationError")]
    SignatureVerificationError {},
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
    #[error("IsNotUser")]
    IsNotUser {},
    #[error("PubKeyIsNotValid")]
    PubKeyIsNotValid {},
    #[error("PubKeyLengthIsNotValid")]
    PubKeyLengthIsNotValid {},
    #[error("NoncesAreNotEqual")]
    NoncesAreNotEqual {},
    #[error("SameCodeId")]
    SameCodeId {},
    #[error("AddressesAreEqual")]
    AddressesAreEqual {},
    #[error("RelayerDoesNotExist")]
    RelayerDoesNotExist {},
}
