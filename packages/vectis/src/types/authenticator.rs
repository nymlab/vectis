use cosmwasm_schema::cw_serde;

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
    pub ty: AuthenticatorType,
    pub provider: AuthenticatorProvider,
}

impl Authenticator {
    pub fn ty(&self) -> &AuthenticatorType {
        &self.ty
    }

    pub fn provider(&self) -> &AuthenticatorProvider {
        &self.provider
    }
}

#[cw_serde]
pub struct EmptyInstantiateMsg {}
