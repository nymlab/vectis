use cosmwasm_std::{CosmosMsg, Event, Response, StdError, StdResult};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Item;
use sylvia::{
    contract, schemars,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};
use vectis_wallet::interface::{post_tx_hook_trait, PostTxHookTrait};

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct PostTxHook<'a> {
    owner: Item<'a, String>,
    counter: Item<'a, u64>,
}

#[contract]
#[messages(post_tx_hook_trait as PostTxHookTrait)]
#[error(StdError)]
impl<'a> PostTxHookTrait for PostTxHook<'a> {
    type Error = StdError;

    #[msg(exec)]
    fn post_tx_hook(&self, ctx: ExecCtx, _msgs: Vec<CosmosMsg>) -> Result<Response, Self::Error> {
        let owner = self.owner.load(ctx.deps.storage)?;
        if ctx.info.sender.as_str() != owner {
            return Err(StdError::GenericErr {
                msg: "error".into(),
            });
        }

        self.counter.update(ctx.deps.storage, |c| -> StdResult<_> {
            Ok(c.checked_add(1).expect("overflow"))
        })?;

        Ok(Response::new())
    }

    #[msg(query)]
    fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError> {
        get_contract_version(ctx.deps.storage)
    }
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(StdError)]
#[messages(post_tx_hook_trait as PostTxHookTrait)]
impl PostTxHook<'_> {
    pub const fn new() -> Self {
        Self {
            owner: Item::new("owner"),
            counter: Item::new("counter"),
        }
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, StdError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        self.owner
            .save(ctx.deps.storage, &ctx.info.sender.to_string())?;
        self.counter.save(ctx.deps.storage, &0)?;
        let event = self.get_event("instantiate");
        Ok(Response::new().add_event(event))
    }

    pub(crate) fn get_event(&self, action: impl Into<String>) -> Event {
        Event::new("vectis.post-tx-test.v1").add_attribute("action", action)
    }

    #[msg(query)]
    fn query_counter(&self, ctx: QueryCtx) -> Result<u64, StdError> {
        self.counter.load(ctx.deps.storage)
    }
}
