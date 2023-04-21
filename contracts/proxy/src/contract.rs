#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response, StdResult, SubMsg,
    WasmMsg,
};
use cw2::set_contract_version;
use vectis_wallet::{WalletCreateReply, DEPLOYER};

use crate::{
    error::ContractError,
    execute::*,
    helpers::{add_plugin_to_state, addresses_to_voters, create_rotate_guardian_factory_msg},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    query::*,
    state::{
        Controller, CODE_ID, CONTROLLER, CREATED_AT, FROZEN, GUARDIANS, LABEL, MULTISIG_ADDRESS,
        PENDING_GUARDIAN_ROTATION, PENDING_MULTISIG, PENDING_PLUGIN, RELAYERS,
    },
    MAX_MULTISIG_VOTING_PERIOD, MULTISIG_INSTANTIATE_ID, MULTISIG_ROTATION_ID, PLUGIN_INST_ID,
    REG_PLUGIN_INST_ID,
};
use cw3_fixed_multisig::msg::InstantiateMsg as FixedMultisigInstantiateMsg;
use cw_utils::{parse_reply_execute_data, parse_reply_instantiate_data, Threshold};

#[cfg(feature = "migration")]
use vectis_wallet::ProxyMigrateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:vectis-proxy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    CREATED_AT.save(deps.storage, &env.block.time.seconds())?;

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
            plugin_permissions,
            migrate_msg,
        } => execute_update_plugin(deps, info, plugin_addr, plugin_permissions, migrate_msg),
        ExecuteMsg::PluginExecute { msgs } => execute_plugin_msgs(deps, info, msgs),
    }
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
            let permissions = PENDING_PLUGIN.load(deps.storage)?;
            add_plugin_to_state(
                deps.storage,
                &permissions,
                &deps.api.addr_canonicalize(&res.contract_address)?,
            )?;
            PENDING_PLUGIN.remove(deps.storage);

            Ok(Response::new().add_attribute("Vectis plugin_address", res.contract_address))
        } else {
            Err(ContractError::PluginInstantiationError {})
        }
    } else if reply.id == REG_PLUGIN_INST_ID {
        if let Ok(res) = parse_reply_execute_data(reply) {
            // All plugins from registry will return this in set_data
            let addr = res.data.ok_or(ContractError::PluginInstantiationError {})?;

            // Add plugins to states
            let permissions = PENDING_PLUGIN.load(deps.storage)?;
            add_plugin_to_state(deps.storage, &permissions, &addr)?;
            PENDING_PLUGIN.remove(deps.storage);

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
        QueryMsg::Plugins {} => to_binary(&query_plugins(deps)?),
    }
}

#[cfg(feature = "migration")]
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: ProxyMigrateMsg) -> Result<Response, ContractError> {
    CODE_ID.save(deps.storage, &msg.new_code_id)?;
    Ok(Response::default())
}
