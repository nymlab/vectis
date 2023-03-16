use crate::types::error::DaoItemsQueryError;
use crate::types::DaoActors;
use cosmwasm_std::Deps;

use crate::types::state::{DAO, ITEMS};

pub fn get_items_from_dao(deps: Deps, item: DaoActors) -> Result<String, DaoItemsQueryError> {
    let dao = deps.api.addr_humanize(
        &DAO.load(deps.storage)
            .map_err(|_| DaoItemsQueryError::DaoAddrNotFound)?,
    )?;
    ITEMS
        .query(&deps.querier, dao, item.to_string())?
        .ok_or(DaoItemsQueryError::ItemNotSet(item.to_string()))
}
