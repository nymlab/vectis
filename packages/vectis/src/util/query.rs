use crate::types::error::DeployerItemsQueryError;
use crate::types::state::VectisActors;
use cosmwasm_std::Deps;

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
