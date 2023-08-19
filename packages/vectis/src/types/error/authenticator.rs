use super::*;

#[derive(Error, Debug, PartialEq)]
pub enum AuthenticatorError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("decoding error {0}")]
    DecodeData(String),
    #[error("Serde")]
    Serde,
    #[error("invalid challenge")]
    InvalidChallenge,
    #[error("signature parsing {0}")]
    SignatureParse(String),
    #[error("pubkey parsing {0}")]
    PubKeyParse(String),
}
