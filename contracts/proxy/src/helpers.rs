use cosmwasm_std::{Addr, CanonicalAddr, Deps, Env, Order, StdResult};
use std::iter::Iterator;

use crate::error::ContractError;
use crate::state::{Controller, CONTROLLER, FROZEN, GUARDIANS, MULTISIG_ADDRESS, RELAYERS};
use cw3_fixed_multisig::msg::Voter;
use cw_storage_plus::Map;

// Converts addresses to voters with weight of 1
pub fn addresses_to_voters(addresses: &[String]) -> Vec<Voter> {
    addresses
        .iter()
        .map(|address| Voter {
            addr: address.to_owned(),
            weight: 1,
        })
        .collect()
}

/// Load addresses from store
pub fn load_addresses(deps: &Deps, addresses: Map<&[u8], ()>) -> StdResult<Vec<Addr>> {
    addresses
        .keys(deps.storage, None, None, Order::Ascending)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key?)))
        .collect()
}

/// Load canonical addresses from store
pub fn load_canonical_addresses(deps: &Deps, addresses: Map<&[u8], ()>) -> StdResult<Vec<Vec<u8>>> {
    addresses
        .keys(deps.storage, None, None, Order::Ascending)
        .collect()
}

/// Ensures sender is guardian
pub fn ensure_is_guardian(deps: Deps, sender: &CanonicalAddr) -> Result<(), ContractError> {
    if is_guardian(deps, sender)? {
        Ok(())
    } else {
        Err(ContractError::IsNotGuardian {})
    }
}

/// Ensures sender is relayed from current contract
pub fn ensure_is_contract_self(env: &Env, sender: &Addr) -> Result<(), ContractError> {
    if sender != &env.contract.address {
        return Err(ContractError::IsNotContractSelf {});
    }
    Ok(())
}

/// Is used to authorize guardian or multisig contract
pub fn authorize_guardian_or_multisig(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    let s = deps.api.addr_canonicalize(sender.as_ref())?;
    match MULTISIG_ADDRESS.may_load(deps.storage)? {
        // if multisig is set, ensure it's address equal to the caller address
        Some(Some(multisig_address)) if multisig_address.eq(&s) => Ok(()),
        Some(_) => Err(ContractError::IsNotMultisig {}),
        // if multisig is not set, ensure caller address is guardian
        _ => ensure_is_guardian(deps, &s),
    }
}

/// Is used to authorize controller or guardians
pub fn authorize_controller_or_guardians(deps: Deps, sender: &Addr) -> Result<(), ContractError> {
    let addr_canonical = deps.api.addr_canonicalize(sender.as_ref())?;
    match MULTISIG_ADDRESS.may_load(deps.storage)? {
        // if multisig adrdess is set, check whether sender is equal to it
        Some(Some(multisig_address)) if multisig_address.eq(&addr_canonical) => Ok(()),
        // otherwise do ucer or guardian auth
        _ => {
            let is_controller_result = ensure_is_controller(deps, sender.as_ref());
            // either guardian or multisig.
            let is_guardian_result = authorize_guardian_or_multisig(deps, sender);
            if is_controller_result.is_ok() || is_guardian_result.is_ok() {
                Ok(())
            } else {
                is_controller_result?;
                is_guardian_result?;
                Ok(())
            }
        }
    }
}

/// Ensures sender is relayer
pub fn ensure_is_relayer(deps: Deps, sender: &CanonicalAddr) -> Result<(), ContractError> {
    if is_relayer(deps, sender)? {
        Ok(())
    } else {
        Err(ContractError::IsNotRelayer {})
    }
}

/// Ensures sender is the wallet controller
pub fn ensure_is_controller(deps: Deps, sender: &str) -> Result<Controller, ContractError> {
    let registered_controller = CONTROLLER.load(deps.storage)?;
    let s = deps.api.addr_canonicalize(sender)?;

    if registered_controller.addr == s {
        Ok(registered_controller)
    } else {
        Err(ContractError::IsNotController {})
    }
}

/// ensure this is either a direct message from the controller
/// or ensure this is relayed by a relayer from this proxy
pub fn ensure_is_relayer_or_controller(
    deps: Deps,
    env: &Env,
    sender: &Addr,
) -> Result<(), ContractError> {
    let is_controller = ensure_is_controller(deps, sender.as_ref());
    let is_contract = ensure_is_contract_self(env, sender);
    if is_controller.is_err() && is_contract.is_err() {
        is_controller?;
        is_contract?;
    };
    Ok(())
}

/// Checks if the sender is a guardian
pub fn is_guardian(deps: Deps, sender: &CanonicalAddr) -> StdResult<bool> {
    GUARDIANS
        .may_load(deps.storage, sender)
        .map(|cfg| cfg.is_some())
}

/// Checks if the sender is a relayer
pub fn is_relayer(deps: Deps, sender: &CanonicalAddr) -> StdResult<bool> {
    RELAYERS
        .may_load(deps.storage, sender)
        .map(|cfg| cfg.is_some())
}

// This is currently just letting guardians execute
pub fn is_frozen(deps: Deps) -> StdResult<bool> {
    FROZEN.load(deps.storage)
}
