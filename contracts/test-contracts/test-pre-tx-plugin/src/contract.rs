use cosmwasm_std::{CosmosMsg, Event, Response, StdError};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use sylvia::{
    contract, schemars,
    types::{InstantiateCtx, QueryCtx},
};
use vectis_wallet::interface::{pre_tx_check_trait, PreTxCheckTrait};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct PreTxCheck {}

#[contract]
#[messages(pre_tx_check_trait as PreTxQueryTrait)]
impl PreTxCheckTrait for PreTxCheck {
    type Error = StdError;

    #[msg(query)]
    fn pre_tx_check(&self, _ctx: QueryCtx, msgs: Vec<CosmosMsg>) -> Result<bool, Self::Error> {
        for msg in msgs {
            if let CosmosMsg::Bank(_) = msg {
                return Ok(false);
            }
        }
        Ok(true)
    }

    #[msg(query)]
    fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError> {
        get_contract_version(ctx.deps.storage)
    }
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(StdError)]
#[messages(pre_tx_check_trait as PreTxCheckTrait)]
impl PreTxCheck {
    pub const fn new() -> Self {
        Self {}
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, StdError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        let event = self.get_event("instantiate");
        Ok(Response::new().add_event(event))
    }

    pub(crate) fn get_event(&self, action: impl Into<String>) -> Event {
        Event::new("vectis.webauthn.v1").add_attribute("action", action)
    }
}
