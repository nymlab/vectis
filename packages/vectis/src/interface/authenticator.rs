use cosmwasm_std::StdError;
use cw2::ContractVersion;
use sylvia::types::QueryCtx;
use sylvia::{interface, schemars};

pub mod authenticator_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait AuthenticatorTrait {
        type Error: From<StdError>;

        #[msg(query)]
        fn authenticate(
            &self,
            ctx: QueryCtx,
            signed_data: Vec<u8>,
            controller_data: Vec<u8>,
            metadata: Vec<Vec<u8>>,
            signature: Vec<u8>,
        ) -> Result<bool, Self::Error>;

        #[msg(query)]
        fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError>;
    }
}
