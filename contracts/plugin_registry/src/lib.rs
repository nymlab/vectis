pub mod contract;
pub mod error;
pub mod responses;

pub const INSTALL_REPLY: u64 = u64::MIN;

#[cfg(any(test, feature = "tests"))]
pub mod multitest;

mod entry_points {
    use crate::contract::{ContractExecMsg, ContractQueryMsg, InstantiateMsg, PluginRegistry};
    use crate::error::ContractError;
    use crate::INSTALL_REPLY;
    use cosmwasm_std::{
        ensure_eq, entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    };
    use cw_utils::parse_reply_instantiate_data;

    const CONTRACT: PluginRegistry = PluginRegistry::new();

    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ContractExecMsg,
    ) -> Result<Response, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env, info))
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: ContractQueryMsg) -> Result<Binary, ContractError> {
        msg.dispatch(&CONTRACT, (deps, env))
    }

    /// reply hooks handles replies from plugin instantiation
    /// `set_data` tells the proxy what its installed plugin address is
    #[entry_point]
    pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
        ensure_eq!(reply.id, INSTALL_REPLY, ContractError::NotSupportedReplyId);
        let data = parse_reply_instantiate_data(reply)?;
        Ok(Response::new().set_data(deps.api.addr_canonicalize(&data.contract_address)?))
    }
}

#[cfg(not(feature = "library"))]
pub use crate::entry_points::*;
