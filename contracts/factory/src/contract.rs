#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use vectis_wallet::factory_queries::*;
use vectis_wallet::{factory_execute, factory_instantiate, factory_reply};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    factory_instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateWallet { create_wallet_msg } => {
            factory_execute::create_wallet(deps, info, env, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            migration_msg,
        } => factory_execute::migrate_wallet(deps, info, wallet_address, migration_msg),
        ExecuteMsg::UpdateCodeId { ty, new_code_id } => {
            factory_execute::update_code_id(deps, info, ty, new_code_id)
        }
        ExecuteMsg::UpdateConfigFee { ty, new_fee } => {
            factory_execute::update_config_fee(deps, info, ty, new_fee)
        }
        ExecuteMsg::UpdateDeployer { addr } => {
            factory_execute::update_deployer_addr(deps, info, addr)
        }
        ExecuteMsg::UpdateController {
            old_controller,
            new_controller,
        } => factory_execute::update_controller(deps, info, old_controller, new_controller),
        ExecuteMsg::UpdateGuardians {
            old_guardians,
            new_guardians,
        } => factory_execute::update_guardians(deps, info, old_guardians, new_guardians),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CodeId { ty } => to_binary(&query_code_id(deps, ty)?),
        QueryMsg::Fees {} => to_binary(&query_fees(deps)?),
        QueryMsg::DeployerAddr {} => to_binary(&query_deployer(deps)?),
        QueryMsg::TotalCreated {} => to_binary(&query_total(deps)?),
        QueryMsg::ControllerWallets { controller } => {
            to_binary(&query_controller_wallets(deps, controller)?)
        }
        QueryMsg::WalletsWithGuardian { guardian } => {
            to_binary(&query_wallets_with_guardian(deps, guardian)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    factory_reply::handle_create_wallet_reply(deps, env, reply)
}
