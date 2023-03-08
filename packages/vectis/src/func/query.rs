use crate::types::error::DaoItemsQueryError;
use crate::types::DaoActors;
use cosmwasm_std::Deps;

#[cfg(not(feature = "test_utils"))]
use crate::types::state::{DAO, ITEMS};

#[cfg(not(feature = "test_utils"))]
pub fn get_items_from_dao(deps: Deps, item: DaoActors) -> Result<String, DaoItemsQueryError> {
    let dao = deps.api.addr_humanize(
        &DAO.load(deps.storage)
            .map_err(|_| DaoItemsQueryError::DaoAddrNotFound)?,
    )?;
    ITEMS
        .query(&deps.querier, dao, item.to_string())?
        .ok_or(DaoItemsQueryError::ItemNotSet(item.to_string()))
}

#[cfg(feature = "test_utils")]
pub fn get_items_from_dao(_deps: Deps, item: DaoActors) -> Result<String, DaoItemsQueryError> {
    Ok(item.to_string())
}