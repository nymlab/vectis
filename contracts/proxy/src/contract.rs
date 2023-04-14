use cosmwasm_schema::schemars;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Order, Reply, Response, StdResult, SubMsg, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use std::fmt;
use vectis_wallet::{
    get_items_from_deployer, pub_key_to_address, query_verify_cosmos, GuardiansUpdateMsg,
    GuardiansUpdateRequest, PluginListResponse, PluginParams, PluginSource, RelayTransaction,
    RelayTxError, VectisActors, WalletCreateReply, WalletFactoryExecuteMsg, WalletInfo,
    DEFAULT_LIMIT, DEPLOYER, MAX_LIMIT,
};

use crate::error::ContractError;
use crate::helpers::{
    addresses_to_voters, authorize_controller_or_guardians, authorize_guardian_or_multisig,
    create_rotate_guardian_factory_msg, ensure_is_contract_self, ensure_is_controller,
    ensure_is_relayer, ensure_is_relayer_or_controller, is_frozen, is_relayer, load_addresses,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    Controller, ADDR_PREFIX, CODE_ID, CONTROLLER, FROZEN, GUARDIANS, LABEL, MULTISIG_ADDRESS,
    PENDING_GUARDIAN_ROTATION, PENDING_MULTISIG, PLUGINS, PROXY_MULTISIG_CODE_ID, RELAYERS,
};
use cw3_fixed_multisig::msg::InstantiateMsg as FixedMultisigInstantiateMsg;
use cw_utils::{parse_reply_execute_data, parse_reply_instantiate_data, Duration, Threshold};
use vectis_plugin_registry::contract::ExecMsg as PluginRegExecMsg;

#[cfg(feature = "migration")]
use vectis_wallet::ProxyMigrateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vectis-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Max voting is set to > 7 years
const MAX_MULTISIG_VOTING_PERIOD: Duration = Duration::Time(2 << 27);

// set resasonobly high value and not interfere with multisigs
/// Used to spot an multisig instantiate reply
const MULTISIG_INSTANTIATE_ID: u64 = u64::MAX;
const MULTISIG_ROTATION_ID: u64 = u64::MAX - 1u64;
const PLUGIN_INST_ID: u64 = u64::MAX - 2u64;
const REG_PLUGIN_INST_ID: u64 = u64::MAX - 3u64;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // Ensure no guardians are the same as a controller
    // https://github.com/nymlab/vectis/issues/43
    let controller_addr_human = deps
        .api
        .addr_validate(&msg.create_wallet_msg.controller_addr)?;
    msg.create_wallet_msg
        .guardians
        .verify_guardians(&controller_addr_human)?;

    let addr = deps.api.addr_canonicalize(controller_addr_human.as_str())?;
    CONTROLLER.save(deps.storage, &Controller { addr, nonce: 0 })?;

    FROZEN.save(deps.storage, &false)?;

    #[cfg(not(test))]
    DEPLOYER.save(deps.storage, &DEPLOYER.query(&deps.querier, info.sender)?)?;

    #[cfg(test)]
    {
        let canon_addr = deps.api.addr_canonicalize("test-DEPLOYER")?;
        DEPLOYER.save(deps.storage, &canon_addr)?;
    }

    CODE_ID.save(deps.storage, &msg.code_id)?;
    LABEL.save(deps.storage, &msg.create_wallet_msg.label)?;

    let guardian_addresses = &msg.create_wallet_msg.guardians.addresses;

    for guardian in guardian_addresses {
        let guardian = deps.api.addr_canonicalize(guardian.as_str())?;
        GUARDIANS.save(deps.storage, &guardian, &())?;
    }

    for relayer in msg.create_wallet_msg.relayers.iter() {
        let relayer = deps.api.addr_canonicalize(relayer)?;
        RELAYERS.save(deps.storage, &relayer, &())?;
    }

    let event = Event::new("vectis.proxy.v1.MsgInstantiate").add_attributes(vec![
        ("controller_address", controller_addr_human.to_string()),
        ("code_id", msg.code_id.to_string()),
        ("label", msg.create_wallet_msg.label),
        ("relayers", format!("{:?}", msg.create_wallet_msg.relayers)),
        ("guardians", format!("{guardian_addresses:?}")),
    ]);

    let mut resp = Response::new().add_event(event);

    // Instantiates a cw3 multisig contract if multisig option is provided for guardians
    if let Some(multisig) = msg.create_wallet_msg.guardians.guardians_multisig {
        PENDING_MULTISIG.save(
            deps.storage,
            &(controller_addr_human.clone(), guardian_addresses.clone()),
        )?;
        let multisig_instantiate_msg = FixedMultisigInstantiateMsg {
            voters: addresses_to_voters(guardian_addresses),
            threshold: Threshold::AbsoluteCount {
                weight: multisig.threshold_absolute_count,
            },
            max_voting_period: MAX_MULTISIG_VOTING_PERIOD,
        };

        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.multisig_code_id,
            msg: to_binary(&multisig_instantiate_msg)?,
            funds: vec![],
            label: "Wallet-Multisig".into(),
        };
        let msg = SubMsg::reply_on_success(instantiate_msg, MULTISIG_INSTANTIATE_ID);
        resp = resp.add_submessage(msg);
    }

    // Data set here will be over written on reply if there is multisig guardian instantiation
    Ok(resp.set_data(to_binary(&WalletCreateReply {
        controller: controller_addr_human,
        proxy_addr: env.contract.address,
        multisig_addr: None,
        guardians: guardian_addresses.clone(),
    })?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Execute { msgs } => execute_execute(deps, info, msgs),
        ExecuteMsg::Relay { transaction } => execute_relay(deps, info, transaction),
        ExecuteMsg::RevertFreezeStatus {} => execute_revert_freeze_status(deps, info),
        ExecuteMsg::RotateControllerKey {
            new_controller_address,
        } => execute_rotate_controller_key(deps, info, new_controller_address),
        ExecuteMsg::AddRelayer {
            new_relayer_address,
        } => execute_add_relayer(deps, info, new_relayer_address),
        ExecuteMsg::RemoveRelayer { relayer_address } => {
            execute_remove_relayer(deps, info, relayer_address)
        }
        ExecuteMsg::RequestUpdateGuardians { request } => {
            execute_request_update_guardians(deps, info, env, request)
        }
        ExecuteMsg::UpdateGuardians {} => execute_update_guardians(deps, env, info),
        ExecuteMsg::UpdateLabel { new_label } => execute_update_label(deps, info, env, new_label),
        ExecuteMsg::InstantiatePlugin {
            src,
            instantiate_msg,
            plugin_params,
            label,
        } => execute_inst_plugin(deps, env, info, src, instantiate_msg, plugin_params, label),
        ExecuteMsg::UpdatePlugins {
            plugin_addr,
            migrate_msg,
        } => execute_update_plugin(deps, info, plugin_addr, migrate_msg),
        ExecuteMsg::PluginExecute { msgs } => execute_plugin_msgs(deps, info, msgs),
    }
}

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
    if plugin_params.has_full_access() {
        let sub_msg = match src {
            PluginSource::VectisRegistry(id) => {
                let registry =
                    get_items_from_deployer(deps.as_ref(), VectisActors::PluginRegistry)?;
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
    } else {
        // Instantiate through grantor contract to get partial access
        Err(ContractError::FeatureNotSupported)
    }
}

/// Add without instantiate, migrate or remove plugin
pub fn execute_update_plugin(
    deps: DepsMut,
    info: MessageInfo,
    plugin_addr: String,
    migrate_msg: Option<(u64, Binary)>,
) -> Result<Response, ContractError> {
    ensure_is_controller(deps.as_ref(), info.sender.as_str())?;
    let addr = deps
        .api
        .addr_canonicalize(deps.api.addr_validate(&plugin_addr)?.as_str())?;
    let res = Response::new().add_attribute("Plugin Addr", &plugin_addr);
    match PLUGINS.may_load(deps.storage, addr.as_slice())? {
        Some(_) => match migrate_msg {
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
            None => {
                PLUGINS.remove(deps.storage, addr.as_slice());
                Ok(res.add_attribute("vectis.proxy.v1/MsgUpdatePlugin", "Remove"))
            }
        },
        None => {
            PLUGINS.save(deps.storage, addr.as_slice(), &())?;
            Ok(res.add_attribute("vectis.proxy.v1/MsgUpdatePlugin", "Add Existing"))
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
    PLUGINS.load(deps.storage, plugin.as_slice())?;
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

    let factory = get_items_from_deployer(deps.as_ref(), VectisActors::Factory)?;
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
            let factory = get_items_from_deployer(deps.as_ref(), VectisActors::Factory)?;
            PROXY_MULTISIG_CODE_ID.query(&deps.querier, Addr::unchecked(factory))?
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    if reply.id == MULTISIG_INSTANTIATE_ID {
        if let Ok(res) = parse_reply_instantiate_data(reply) {
            MULTISIG_ADDRESS.save(
                deps.storage,
                &Some(deps.api.addr_canonicalize(&res.contract_address)?),
            )?;

            let (controller, guardians) = PENDING_MULTISIG.load(deps.storage)?;
            PENDING_MULTISIG.remove(deps.storage);

            let data = to_binary(&WalletCreateReply {
                controller,
                proxy_addr: env.contract.address,
                multisig_addr: Some(deps.api.addr_validate(&res.contract_address)?),
                guardians,
            })?;
            let event = Event::new("vectis.proxy.v1.MsgReplyMultisigInstantiate")
                .add_attribute("multisig_address", res.contract_address);

            Ok(Response::new().add_event(event).set_data(data))
        } else {
            Err(ContractError::MultisigInstantiationError {})
        }
    } else if reply.id == MULTISIG_ROTATION_ID {
        // reply on success so can unwrap
        let data = parse_reply_instantiate_data(reply).unwrap();
        MULTISIG_ADDRESS.save(
            deps.storage,
            &Some(deps.api.addr_canonicalize(&data.contract_address)?),
        )?;

        let request = PENDING_GUARDIAN_ROTATION.load(deps.storage)?;

        let factory_msg = create_rotate_guardian_factory_msg(
            deps.as_ref(),
            request.old_guardians,
            request.new_guardians.addresses,
        )?;
        Ok(Response::new().add_submessage(factory_msg))
    } else if reply.id == PLUGIN_INST_ID {
        if let Ok(res) = parse_reply_instantiate_data(reply) {
            PLUGINS.save(
                deps.storage,
                deps.api
                    .addr_canonicalize(&res.contract_address)?
                    .as_slice(),
                &(),
            )?;

            Ok(Response::new()
                .add_attribute("action", "Plugin Stored")
                .add_attribute("plugin_address", res.contract_address))
        } else {
            Err(ContractError::PluginInstantiationError {})
        }
    } else if reply.id == REG_PLUGIN_INST_ID {
        if let Ok(res) = parse_reply_execute_data(reply) {
            let addr = res.data.ok_or(ContractError::PluginInstantiationError {})?;
            PLUGINS.save(deps.storage, &addr, &())?;

            Ok(Response::new()
                .add_attribute("action", "Plugin Stored")
                .add_attribute(
                    "plugin_address",
                    deps.api.addr_humanize(&addr.into())?.into_string(),
                ))
        } else {
            Err(ContractError::PluginInstantiationExecError {})
        }
    } else {
        Err(ContractError::InvalidMessage {
            msg: "invalid ID".to_string(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::CanExecuteRelay { sender } => to_binary(&query_can_execute_relay(deps, sender)?),
        QueryMsg::GuardiansUpdateRequest {} => to_binary(&query_guardian_update_request(deps)?),
        QueryMsg::Plugins { start_after, limit } => {
            to_binary(&query_plugins(deps, start_after, limit)?)
        }
    }
}

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

pub fn query_plugins(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<PluginListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let items = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            let start = Some(Bound::exclusive(wallet_addr.as_slice()));
            PLUGINS
                .prefix(())
                .range(deps.storage, start, None, Order::Ascending)
        }
        None => PLUGINS
            .prefix(())
            .range(deps.storage, None, None, Order::Ascending),
    };

    let plugins: StdResult<Vec<Addr>> = items
        .take(limit)
        .map(|w| -> StdResult<Addr> {
            let ww = w?;
            deps.api.addr_humanize(&CanonicalAddr::from(ww.0))
        })
        .collect();

    Ok(PluginListResponse { plugins: plugins? })
}

#[cfg(feature = "migration")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: ProxyMigrateMsg) -> Result<Response, ContractError> {
    CODE_ID.save(deps.storage, &msg.new_code_id)?;
    Ok(Response::default())
}
