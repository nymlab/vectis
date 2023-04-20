use cosmwasm_std::{Addr, CanonicalAddr, Deps, Order, StdResult};
use cw1::CanExecuteResponse;
use cw_storage_plus::Bound;
use vectis_wallet::{
    GuardiansUpdateRequest, PluginListResponse, PluginPermissions, WalletInfo, DEFAULT_LIMIT,
    DEPLOYER, MAX_LIMIT,
};

use crate::helpers::{is_relayer, load_addresses};
use crate::state::{
    CODE_ID, CONTROLLER, EXEC_PLUGINS, FROZEN, GUARDIANS, LABEL, MULTISIG_ADDRESS, MULTISIG_PLUGIN,
    PENDING_GUARDIAN_ROTATION, PRE_TX_PLUGINS, QUERY_PLUGINS, RELAYERS,
};

/// Returns wallet info
pub fn query_info(deps: Deps) -> StdResult<WalletInfo> {
    let guardians = load_addresses(&deps, GUARDIANS)?;
    let relayers = load_addresses(&deps, RELAYERS)?;
    let multisig_address = match MULTISIG_ADDRESS.may_load(deps.storage)? {
        Some(Some(c)) => Some(deps.api.addr_humanize(&c)?),
        _ => None,
    };

    let controller = CONTROLLER.load(deps.storage)?;

    Ok(WalletInfo {
        controller_addr: deps.api.addr_humanize(&controller.addr)?,
        deployer: deps.api.addr_humanize(&DEPLOYER.load(deps.storage)?)?,
        nonce: controller.nonce,
        version: cw2::get_contract_version(deps.storage)?,
        code_id: CODE_ID.load(deps.storage)?,
        guardians,
        relayers,
        is_frozen: FROZEN.load(deps.storage)?,
        multisig_address,
        // TODO
        multisig_threshold: None,
        label: LABEL.load(deps.storage)?,
    })
}

/// Returns if sender is a relayer
pub fn query_can_execute_relay(deps: Deps, sender: String) -> StdResult<CanExecuteResponse> {
    let sender_canonical = deps.api.addr_canonicalize(&sender)?;
    Ok(CanExecuteResponse {
        can_execute: is_relayer(deps, &sender_canonical)?,
    })
}

pub fn query_guardian_update_request(deps: Deps) -> StdResult<Option<GuardiansUpdateRequest>> {
    PENDING_GUARDIAN_ROTATION.may_load(deps.storage)
}

pub fn query_plugins(deps: Deps) -> StdResult<PluginListResponse> {
    let exec_plugins = EXEC_PLUGINS
        .prefix(())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|w| -> StdResult<Addr> {
            let ww = w?;
            deps.api.addr_humanize(&CanonicalAddr::from(ww.0))
        })
        .collect::<StdResult<Vec<Addr>>>()?;
    let query_plugins = QUERY_PLUGINS
        .prefix(())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|w| -> StdResult<Addr> {
            let ww = w?;
            deps.api.addr_humanize(&CanonicalAddr::from(ww.1))
        })
        .collect::<StdResult<Vec<Addr>>>()?;
    let pre_tx_plugins = PRE_TX_PLUGINS
        .prefix(())
        .range(deps.storage, None, None, Order::Ascending)
        .map(|w| -> StdResult<Addr> {
            let ww = w?;
            deps.api.addr_humanize(&CanonicalAddr::from(ww.0))
        })
        .collect::<StdResult<Vec<Addr>>>()?;

    let multisig_override = MULTISIG_PLUGIN
        .may_load(deps.storage)?
        .map(|a| deps.api.addr_humanize(&CanonicalAddr::from(a)))
        .transpose()?;

    Ok(PluginListResponse {
        exec_plugins,
        query_plugins,
        pre_tx_plugins,
        multisig_override,
    })
}
