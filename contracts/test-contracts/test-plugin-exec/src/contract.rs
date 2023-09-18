use cosmwasm_std::{to_binary, CosmosMsg, Event, Response, StdError};
use cw2::{get_contract_version, set_contract_version, ContractVersion};
use cw_storage_plus::Item;
use sylvia::{
    contract, schemars,
    types::{ExecCtx, InstantiateCtx, QueryCtx},
};

// Vectis lib
use vectis_wallet::interface::wallet_plugin_trait::ExecMsg as VectisWalletExecMsg;

#[cfg(not(feature = "library"))]
use sylvia::entry_points;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct PluginExec<'a> {
    owner: Item<'a, String>,
}

#[cfg_attr(not(feature = "library"), entry_points)]
#[contract]
#[error(StdError)]
impl PluginExec<'_> {
    pub const fn new() -> Self {
        Self {
            owner: Item::new("owner"),
        }
    }

    #[msg(instantiate)]
    fn instantiate(&self, ctx: InstantiateCtx) -> Result<Response, StdError> {
        set_contract_version(ctx.deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        self.owner
            .save(ctx.deps.storage, &ctx.info.sender.to_string())?;

        let event = self.get_event("instantiate");
        Ok(Response::new().add_event(event))
    }

    #[msg(exec)]
    fn exec(&self, ctx: ExecCtx, msgs: Vec<CosmosMsg>) -> Result<Response, StdError> {
        let owner = self.owner.load(ctx.deps.storage)?;

        let vectis_msg = CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: owner,
            msg: to_binary(&VectisWalletExecMsg::PluginExecute { msg: msgs })?,
            funds: vec![],
        });

        Ok(Response::new().add_message(vectis_msg))
    }

    pub(crate) fn get_event(&self, action: impl Into<String>) -> Event {
        Event::new("vectis.plugin-exec-test.v1").add_attribute("action", action)
    }

    #[msg(query)]
    fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError> {
        get_contract_version(ctx.deps.storage)
    }
}
