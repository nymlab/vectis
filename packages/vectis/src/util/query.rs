use crate::types::error::DeployerItemsQueryError;
use crate::types::state::VectisActors;
use cosmos_sdk_proto::{
    ibc::applications::interchain_accounts::controller::v1::QueryInterchainAccountRequest,
    traits::{MessageExt, TypeUrl},
};
use cosmwasm_std::{to_binary, Deps, QueryRequest, StdError};

use crate::types::state::{DEPLOYER, ITEMS};

pub fn get_items_from_deployer(
    deps: Deps,
    item: VectisActors,
) -> Result<String, DeployerItemsQueryError> {
    let deployer = deps.api.addr_humanize(
        &DEPLOYER
            .load(deps.storage)
            .map_err(|_| DeployerItemsQueryError::DeployerAddrNotFound)?,
    )?;
    ITEMS
        .query(&deps.querier, deployer, item.to_string())?
        .ok_or(DeployerItemsQueryError::ItemNotSet(item.to_string()))
}

pub fn get_ica(deps: Deps, owner: String, connection_id: String) -> Result<String, StdError> {
    deps.querier.query(&QueryRequest::Stargate {
        path: QueryInterchainAccountRequest::TYPE_URL.into(),
        data: to_binary(
            &QueryInterchainAccountRequest {
                owner,
                connection_id,
            }
            .to_bytes()
            .map_err(|e| StdError::generic_err(e.to_string()))?,
        )?,
    })
}
