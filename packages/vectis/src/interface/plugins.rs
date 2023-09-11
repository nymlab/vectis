use cosmwasm_std::{CosmosMsg, Response, StdError};
use cw2::ContractVersion;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

pub mod pre_tx_check_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait PreTxCheckTrait {
        type Error: From<StdError>;

        // TODO: maybe ref to vec?
        #[msg(query)]
        fn pre_tx_check(&self, ctx: QueryCtx, msgs: Vec<CosmosMsg>) -> Result<bool, Self::Error>;

        #[msg(query)]
        fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError>;
    }
}

pub mod post_tx_hook_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait PostTxHookTrait {
        type Error: From<StdError>;

        #[msg(exec)]
        fn post_tx_hook(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg>)
            -> Result<Response, Self::Error>;

        #[msg(query)]
        fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError>;
    }
}
