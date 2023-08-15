use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdError;
use sylvia::types::QueryCtx;
use sylvia::{interface, schemars};
use thiserror::Error;

pub mod authenicator_export {
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

    /// The trait for each authenticator contract
    #[interface]
    pub trait AuthenicatorExport {
        type Error: From<StdError>;

        // TODO: how can we make these all &[u8]?
        // #[interface] complains
        #[msg(query)]
        fn authenticate(
            &self,
            ctx: QueryCtx,
            signed_data: Vec<u8>,
            controller_data: Vec<u8>,
            metadata: Vec<Vec<u8>>,
            signature: Vec<u8>,
        ) -> Result<bool, Self::Error>;
    }
}

/// User can decide if they want a different authenticator instead of the Vectis one
#[cw_serde]
pub enum AuthenticatorProvider {
    /// User would like to use Vectis provided authenticator
    Vectis,
    /// User would like to use their own authenticator at this given contract address
    Custom(String),
}

/// Authenticator type maps the authentication method for the main Controller messages
#[cw_serde]
pub enum AuthenticatorType {
    CosmosEOA,
    EthereumEOA,
    Webauthn,
    /// This is for future extensibility without neccessarily upgrading the enum type
    /// It should be the name of the authenticator (i.e. AnonCreds)
    Other(String),
}

impl std::fmt::Display for AuthenticatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Authenticator for the controller
#[cw_serde]
pub struct Authenticator {
    ty: AuthenticatorType,
    provider: AuthenticatorProvider,
}

impl Authenticator {
    pub fn ty(&self) -> &AuthenticatorType {
        &self.ty
    }

    pub fn provider(&self) -> &AuthenticatorProvider {
        &self.provider
    }
}
