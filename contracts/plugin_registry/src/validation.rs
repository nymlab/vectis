use cosmwasm_std::{CanonicalAddr, Deps};

use crate::error::ContractError;

pub fn ensure_is_reviewer(
    deps: Deps,
    reviewers: &Vec<CanonicalAddr>,
    reviewer: &str,
) -> Result<(), ContractError> {
    if !reviewers.contains(&deps.api.addr_canonicalize(reviewer)?) {
        return Err(ContractError::Unauthorized);
    }
    Ok(())
}
