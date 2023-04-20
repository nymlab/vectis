use cosmwasm_schema::schemars;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response, StdResult,
    SubMsg, WasmMsg,
};
use std::fmt;
use vectis_wallet::{
    get_items_from_deployer, pub_key_to_address, query_verify_cosmos, GuardiansUpdateMsg,
    GuardiansUpdateRequest, PluginParams, PluginPermissions, PluginSource, RelayTransaction,
    RelayTxError, VectisActors, WalletFactoryExecuteMsg,
};

use crate::{
    error::ContractError,
    helpers::{
        add_plugin_to_state, addresses_to_voters, authorize_controller_or_guardians,
        authorize_guardian_or_multisig, create_rotate_guardian_factory_msg,
        ensure_is_contract_self, ensure_is_controller, ensure_is_relayer,
        ensure_is_relayer_or_controller, is_frozen, load_addresses,
    },
    state::{
        ADDR_PREFIX, CONTROLLER, EXEC_PLUGINS, FROZEN, GUARDIANS, LABEL, MULTISIG_ADDRESS,
        PENDING_GUARDIAN_ROTATION, PENDING_PLUGIN, PROXY_MULTISIG_CODE_ID, RELAYERS,
    },
    MAX_MULTISIG_VOTING_PERIOD, MULTISIG_ROTATION_ID, PLUGIN_INST_ID, REG_PLUGIN_INST_ID,
};
use cw3_fixed_multisig::msg::InstantiateMsg as FixedMultisigInstantiateMsg;
use cw_utils::Threshold;
use vectis_plugin_registry::contract::ExecMsg as PluginRegExecMsg;

/// Executes instantiation of plugin
pub fn execute_inst_plugin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    src: PluginSource,
    msg: Binary,
    plugin_params: PluginParams,
    label: String,
) -> Result<Response, ContractError> {
    ensure_is_controller(deps.as_ref(), info.sender.as_str())?;

    PENDING_PLUGIN.save(deps.storage, &plugin_params.permissions)?;
    let sub_msg = match src {
        PluginSource::VectisRegistry(id) => {
            let registry = get_items_from_deployer(deps.as_ref(), VectisActors::PluginRegistry)?;
            SubMsg::reply_always(
                WasmMsg::Execute {
                    contract_addr: registry,
                    msg: to_binary(&PluginRegExecMsg::ProxyInstallPlugin {
                        id,
                        instantiate_msg: msg,
                    })?,
                    funds: info.funds,
                },
                REG_PLUGIN_INST_ID,
            )
        }
        PluginSource::CodeId(code_id) => SubMsg::reply_always(
            WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id,
                msg,
                funds: info.funds,
                label,
            },
            PLUGIN_INST_ID,
        ),
    };
    Ok(Response::new().add_submessage(sub_msg))
}

/// Add without instantiate, migrate or remove plugin
pub fn execute_update_plugin(
    deps: DepsMut,
    info: MessageInfo,
    plugin_addr: String,
    plugin_permissions: Option<Vec<PluginPermissions>>,
    migrate_msg: Option<(u64, Binary)>,
) -> Result<Response, ContractError> {
    ensure_is_controller(deps.as_ref(), info.sender.as_str())?;
    let res = Response::new().add_attribute("Plugin Addr", &plugin_addr);

    match migrate_msg {
        // If there is migration msg, then we will assume the contract admin of the plugin is this
        // contract
        Some((new_code_id, msg)) => {
            let wasm_msg = WasmMsg::Migrate {
                contract_addr: plugin_addr,
                new_code_id,
                msg,
            };

            Ok(res
                .add_message(wasm_msg)
                .add_attribute("vectis.proxy.v1/MsgUpdatePlugin", "Migrate"))
        }
        // This means we are either removing the plugin or adding it
        None => {
            let canon_addr = deps.api.addr_canonicalize(&plugin_addr)?;
            match plugin_permissions {
                // we see if it exist in the list, if it does, we remove it
                Some(permissions) => {
                    add_plugin_to_state(deps.storage, &permissions, &canon_addr)?;
                    Ok(res.add_attribute("vectis.proxy.v1/MsgUpdatePlugin", "Add existing"))
                }
                // TODO: we remove plugin_addr from the list
                None => Err(ContractError::FeatureNotSupported),
            }
        }
    }
}

/// Call by plugins
pub fn execute_plugin_msgs(
    deps: DepsMut,
    info: MessageInfo,
    msgs: Vec<CosmosMsg>,
) -> Result<Response, ContractError> {
    let plugin = deps.api.addr_canonicalize(info.sender.as_str())?;
    EXEC_PLUGINS.load(deps.storage, plugin.as_slice())?;
    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("vectis.proxy.v1/PluginExecMsg", info.sender))
}

pub fn execute_execute<T>(
    deps: DepsMut,
    info: MessageInfo,
    msgs: Vec<CosmosMsg<T>>,
) -> Result<Response<T>, ContractError>
where
    T: Clone + fmt::Debug + PartialEq + schemars::JsonSchema,
{
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    // Ensure controller exists
    ensure_is_controller(deps.as_ref(), info.sender.as_ref())?;

    let event = Event::new("vectis.proxy.v1.MsgExecute");

    Ok(Response::new().add_messages(msgs).add_event(event))
}

/// Executes relayed message fro a relayer
pub fn execute_relay(
    deps: DepsMut,
    info: MessageInfo,
    transaction: RelayTransaction,
) -> Result<Response, ContractError> {
    // Ensure sender is a relayer
    let relayer_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;
    ensure_is_relayer(deps.as_ref(), &relayer_addr)?;

    // make sure guardians have not frozen the contract
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    let factory = get_items_from_deployer(deps.as_ref(), VectisActors::Factory)?;
    // Get controller addr from it's pubkey
    let addr = pub_key_to_address(
        &deps.as_ref(),
        &ADDR_PREFIX.query(&deps.querier, Addr::unchecked(factory))?,
        &transaction.controller_pubkey.0,
    )?;

    // Ensure address derived from pub message is the address of existing controller
    let controller = ensure_is_controller(deps.as_ref(), addr.as_ref())?;

    // Ensure relayer provided nonce is correct
    controller.ensure_nonces_are_equal(&transaction.nonce)?;

    // Checks signature, which includes a message including the nonce signed by the controller
    let is_verified = query_verify_cosmos(&deps, &transaction)?;

    if is_verified {
        let msg: Result<CosmosMsg, _> = cosmwasm_std::from_slice(transaction.message.0.as_slice());
        if let Ok(msg) = msg {
            // Update nonce
            CONTROLLER.update(deps.storage, |mut controller| -> StdResult<_> {
                controller.increment_nonce();
                Ok(controller)
            })?;

            let event = Event::new("vectis.proxy.v1.MsgRelay");

            Ok(Response::new().add_message(msg).add_event(event))
        } else {
            Err(ContractError::InvalidMessage {
                msg: msg.unwrap_err().to_string(),
            })
        }
    } else {
        Err(ContractError::RelayTxError(
            RelayTxError::SignatureVerificationError {},
        ))
    }
}

/// Add relayer to the relayers set
pub fn execute_add_relayer(
    deps: DepsMut,
    info: MessageInfo,
    relayer_addr: Addr,
) -> Result<Response, ContractError> {
    // Authorize controller or guardians
    authorize_controller_or_guardians(deps.as_ref(), &info.sender)?;

    // Save a new relayer if it does not exist yet
    let relayer_addr_canonical = deps.api.addr_canonicalize(relayer_addr.as_ref())?;

    if !RELAYERS.has(deps.storage, &relayer_addr_canonical) {
        RELAYERS.save(deps.storage, &relayer_addr_canonical, &())?;
        let event =
            Event::new("vectis.proxy.v1.MsgAddRelayer").add_attribute("address", relayer_addr);

        Ok(Response::new().add_event(event))
    } else {
        Err(ContractError::RelayerAlreadyExists {})
    }
}

/// Remove relayer from the relayers set
pub fn execute_remove_relayer(
    deps: DepsMut,
    info: MessageInfo,
    relayer_addr: Addr,
) -> Result<Response, ContractError> {
    // Authorize controller or guardians
    authorize_controller_or_guardians(deps.as_ref(), &info.sender)?;

    // Remove a relayer if possible
    let relayer_addr_canonical = deps.api.addr_canonicalize(relayer_addr.as_ref())?;

    if RELAYERS.has(deps.storage, &relayer_addr_canonical) {
        RELAYERS.remove(deps.storage, &relayer_addr_canonical);
        let event =
            Event::new("vectis.proxy.v1.MsgRemoveRelayer").add_attribute("address", relayer_addr);
        Ok(Response::new().add_event(event))
    } else {
        Err(ContractError::RelayerDoesNotExist {})
    }
}

/// Change current freezing status to its inverse
/// Must be from a guardian or a guardian multisig contract
pub fn execute_revert_freeze_status(
    deps: DepsMut,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // Ensure caller is guardian or multisig
    authorize_guardian_or_multisig(deps.as_ref(), &info.sender)?;

    // Invert frozen status
    let frozen = FROZEN.update(deps.storage, |mut frozen| -> StdResult<_> {
        frozen ^= true;
        Ok(frozen)
    })?;

    let event = Event::new("vectis.proxy.v1.MsgRevertFreezeStatus")
        .add_attribute("status", if frozen { "frozen" } else { "unfrozen" });

    Ok(Response::new().add_event(event))
}

/// Complete controller key rotation by changing it's address
/// Must be from a guardian or a guardian multisig contract
pub fn execute_rotate_controller_key(
    deps: DepsMut,
    info: MessageInfo,
    new_controller_address: String,
) -> Result<Response, ContractError> {
    // Allow guardians to rotate key when it is frozen
    if is_frozen(deps.as_ref())?
        && authorize_guardian_or_multisig(deps.as_ref(), &info.sender).is_err()
    {
        return Err(ContractError::Frozen {});
    } else {
        authorize_controller_or_guardians(deps.as_ref(), &info.sender)?
    };

    let controller = CONTROLLER.load(deps.storage)?;

    // Ensure provided address is different from current
    let validated_new_controller = deps.api.addr_validate(&new_controller_address)?;
    let new_canon_addr = deps
        .api
        .addr_canonicalize(validated_new_controller.as_str())?;
    controller.ensure_addresses_are_not_equal(&new_canon_addr)?;

    // Update controller address
    CONTROLLER.update(deps.storage, |mut controller| -> StdResult<_> {
        controller.set_address(new_canon_addr);
        Ok(controller)
    })?;

    #[cfg(not(test))]
    let factory = get_items_from_deployer(deps.as_ref(), VectisActors::Factory)?;
    #[cfg(test)]
    let factory = String::from("factory");

    let update_factory_msg = SubMsg::new(WasmMsg::Execute {
        contract_addr: factory,
        msg: to_binary(&WalletFactoryExecuteMsg::UpdateController {
            old_controller: deps.api.addr_humanize(&controller.addr)?,
            new_controller: validated_new_controller,
        })?,
        funds: vec![],
    });

    let event = Event::new("vectis.proxy.v1.MsgRotateControllerKey")
        .add_attribute("old_address", deps.api.addr_humanize(&controller.addr)?)
        .add_attribute("new_address", new_controller_address);

    Ok(Response::new()
        .add_event(event)
        .add_submessage(update_factory_msg))
}

pub fn execute_update_guardians(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    ensure_is_relayer_or_controller(deps.as_ref(), &env, &info.sender)?;

    // make sure guardians have not frozen the contract
    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }

    let request = PENDING_GUARDIAN_ROTATION
        .may_load(deps.storage)?
        .ok_or(ContractError::GuardianRequestNotFound {})?;

    if !request.activate_at.is_expired(&env.block) {
        return Err(ContractError::GuardianRequestNotExecutable {});
    }

    let GuardiansUpdateRequest {
        old_guardians,
        new_guardians,
        new_multisig_code_id,
        ..
    } = request;

    // Replace the entire locally stored guardians list
    for guardian in &old_guardians {
        GUARDIANS.remove(
            deps.storage,
            &deps.api.addr_canonicalize(guardian.as_str())?,
        );
    }
    for guardian in &new_guardians.addresses {
        GUARDIANS.save(
            deps.storage,
            &deps.api.addr_canonicalize(guardian.as_str())?,
            &(),
        )?;
    }

    let mut event = Event::new("vectis.proxy.v1.MsgUpdateGuardians")
        .add_attribute("guardians", format!("{:?}", new_guardians.addresses));

    if let Some(multisig_settings) = new_guardians.guardians_multisig {
        // If new guardian is multisig, we instantiate a new multisig
        // We handle factory update and tmp state in the `reply`
        let instantiation_code_id = if let Some(id) = new_multisig_code_id {
            id
        } else {
            #[cfg(not(test))]
            {
                let factory = get_items_from_deployer(deps.as_ref(), VectisActors::Factory)?;
                PROXY_MULTISIG_CODE_ID.query(&deps.querier, Addr::unchecked(factory))?
            }

            #[cfg(test)]
            {
                1
            }
        };
        let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
            voters: addresses_to_voters(&new_guardians.addresses),
            threshold: Threshold::AbsoluteCount {
                weight: multisig_settings.threshold_absolute_count,
            },
            max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
        };

        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: instantiation_code_id,
            msg: to_binary(&multisig_instantiate_msg)?,
            funds: vec![],
            label: "Wallet-Multisig".into(),
        };
        let msg = SubMsg::reply_on_success(instantiate_msg, MULTISIG_ROTATION_ID);

        event = event
            .add_attribute("multisig", "true")
            .add_attribute("multisig_code_id", instantiation_code_id.to_string());

        Ok(Response::new().add_submessage(msg).add_event(event))
    } else {
        // If new guardians is not a multisig, we are done
        // Call factory for update and remove tmp state
        let factory_msg = create_rotate_guardian_factory_msg(
            deps.as_ref(),
            old_guardians,
            new_guardians.addresses,
        )?;
        MULTISIG_ADDRESS.save(deps.storage, &None)?;
        PENDING_GUARDIAN_ROTATION.remove(deps.storage);

        event = event.add_attribute("multisig", "false");
        Ok(Response::new().add_event(event).add_submessage(factory_msg))
    }
}

pub fn execute_request_update_guardians(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    request: Option<GuardiansUpdateMsg>,
) -> Result<Response, ContractError> {
    ensure_is_relayer_or_controller(deps.as_ref(), &env, &info.sender)?;

    if is_frozen(deps.as_ref())? {
        return Err(ContractError::Frozen {});
    }
    match request {
        // This is to create a request update
        Some(r) => {
            r.guardians.verify_guardians(
                &deps
                    .api
                    .addr_humanize(&CONTROLLER.load(deps.storage)?.addr)?,
            )?;

            let old_guardians = load_addresses(&deps.as_ref(), GUARDIANS)?;

            PENDING_GUARDIAN_ROTATION.save(
                deps.storage,
                &GuardiansUpdateRequest::new(
                    old_guardians,
                    r.guardians.clone(),
                    r.new_multisig_code_id,
                    &env.block,
                ),
            )?;

            let event = Event::new("vectis.proxy.v1.MsgRequestUpdateGuardians")
                .add_attribute("create", "true")
                .add_attribute("guardians", format!("{:?}", r.guardians.addresses));

            Ok(Response::new().add_event(event))
        }
        None => {
            // This is to remove a request update
            PENDING_GUARDIAN_ROTATION.remove(deps.storage);
            let event = Event::new("vectis.proxy.v1.MsgRequestUpdateGuardians")
                .add_attribute("create", "false");
            Ok(Response::new().add_event(event))
        }
    }
}

/// Update label by controller
pub fn execute_update_label(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    new_label: String,
) -> Result<Response, ContractError> {
    let is_controller = ensure_is_controller(deps.as_ref(), info.sender.as_str());
    let is_contract = ensure_is_contract_self(&env, &info.sender);
    if is_controller.is_err() && is_contract.is_err() {
        is_controller?;
        is_contract?;
    }

    LABEL.update(deps.storage, |l| {
        if l == new_label {
            Err(ContractError::SameLabel {})
        } else {
            Ok(new_label.clone())
        }
    })?;

    let event = Event::new("vectis.proxy.v1.MsgUpdateLabel").add_attribute("label", new_label);

    Ok(Response::default().add_event(event))
}
