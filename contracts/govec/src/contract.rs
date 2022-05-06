#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult,
    SubMsg, Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20Coin, Cw20ReceiveMsg, MinterResponse, TokenInfoResponse};
use stake_cw20::msg::InstantiateMsg as StakeInstantiateMsg;

use crate::enumerable::query_all_accounts;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{MinterData, TokenInfo, BALANCES, DAO_ADDR, STAKING_ADDR, TOKEN_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:govec";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const STAKING_REPLY_ID: u64 = u64::MAX;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;
    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    // store minter info
    let minter = match msg.minter {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    // votes cannot be split, therefore decimals is 0
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: 0,
        total_supply,
        minter,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    // store DAO contract addr
    DAO_ADDR.save(deps.storage, &info.sender)?;

    let response = Response::new();

    if let Some(staking_options) = msg.staking {
        // Instantiate Staking
        let staking_inst_msg = StakeInstantiateMsg {
            admin: Some(deps.api.addr_humanize(&msg.dao)?),
            token_address: env.contract.address,
            unstaking_duration: staking_options.duration,
        };

        return Ok(response.add_submessage(SubMsg::reply_always(
            WasmMsg::Instantiate {
                admin: Some(deps.api.addr_humanize(&msg.dao)?.to_string()),
                code_id: staking_options.code_id,
                msg: to_binary(&staking_inst_msg)?,
                funds: vec![],
                label: "Vectis Staking contract".into(),
            },
            STAKING_REPLY_ID,
        )));
    }
    Ok(response)
}

pub fn create_accounts(deps: &mut DepsMut, accounts: &[Cw20Coin]) -> StdResult<Uint128> {
    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount)?;
        total_supply += row.amount;
    }
    Ok(total_supply)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, StdError> {
    if reply.id != STAKING_REPLY_ID {
        return Err(StdError::generic_err("Unsupported reply id"));
    }

    let data = reply.result.into_result().map_err(StdError::generic_err)?;
    let staking_instantiation_event = data
        .events
        .iter()
        .find(|e| e.ty == "instantiate")
        .ok_or_else(|| StdError::generic_err("Reply: Unable to find instantiate event"))?;

    let staking_addr_str = &staking_instantiation_event.attributes[0].value;
    STAKING_ADDR.save(deps.storage, &deps.api.addr_canonicalize(staking_addr_str)?)?;

    let res = Response::new()
        .add_attribute("action", "Staking Deployed")
        .add_attribute("Staking", staking_addr_str);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn {} => execute_burn(deps, env, info),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { new_wallet } => execute_mint(deps, env, info, new_wallet),
        ExecuteMsg::UpdateStakingAddr { new_addr } => execute_update_staking(deps, info, new_addr),
        ExecuteMsg::UpdateMintData { new_mint } => execute_update_mint_data(deps, info, new_mint),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    ensure_is_staking_or_wallet(deps.as_ref(), &rcpt_addr)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Burning of the vote, this can only be used by the approved list of SCW
///
/// Only exactly 1 vote can be burnt per wallet and is executed during destroy of the wallet,
/// the wallet must also only have exactly 1 vote in its balance
pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let to_burn = Uint128::from(1u8);
    // Ensure only have voting power of exactly 1
    let balance = query_balance(deps.as_ref(), info.sender.to_string())?;
    if balance.balance != to_burn {
        return Err(ContractError::IncorrectBalance(balance.balance));
    };

    // remove key from the map as they exit the DAO
    BALANCES.remove(deps.storage, &info.sender);

    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(to_burn)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", to_burn);
    Ok(res)
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_wallet: String,
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO.load(deps.storage)?;
    if config.mint.is_none() || config.mint.as_ref().unwrap().minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += Uint128::from(1u8);
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&new_wallet)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + Uint128::from(1u8))
        },
    )?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", new_wallet)
        .add_attribute("amount", "1");
    Ok(res)
}

/// Send can only be to the staking contract
/// For delegation, use transfer instead where only wallets are in the whitelist
pub fn execute_send(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let contract = deps.api.addr_validate(&contract)?;
    ensure_is_staking_or_wallet(deps.as_ref(), &contract)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &contract,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) },
    )?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(
            Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

pub fn execute_update_staking(
    deps: DepsMut,
    info: MessageInfo,
    new_addr: String,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender)?;

    STAKING_ADDR.save(deps.storage, &deps.api.addr_canonicalize(&new_addr)?)?;

    let res = Response::new()
        .add_attribute("action", "update_staking_address")
        .add_attribute("new_addr", new_addr);

    Ok(res)
}

pub fn execute_update_mint_data(
    deps: DepsMut,
    info: MessageInfo,
    new_mint: Option<MinterData>,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender)?;

    TOKEN_INFO.update(deps.storage, |mut t| -> StdResult<_> {
        t.mint = new_mint.clone();
        Ok(t)
    })?;

    let m = new_mint.unwrap_or(MinterData {
        minter: Addr::unchecked(""),
        cap: None,
    });
    let res = Response::new()
        .add_attribute("action", "update_minter_data")
        .add_attribute("new_minter", m.minter)
        .add_attribute("cap", m.cap.unwrap_or_default());

    Ok(res)
}

fn ensure_is_dao(deps: Deps, sender: Addr) -> Result<(), ContractError> {
    let dao = DAO_ADDR.load(deps.storage)?;
    if dao != deps.api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

fn ensure_is_staking_or_wallet(deps: Deps, contract: &Addr) -> Result<(), ContractError> {
    let staking = STAKING_ADDR.may_load(deps.storage)?;
    let wallet = BALANCES.may_load(deps.storage, contract)?;
    if let Some(staking_addr) = staking {
        if contract == &deps.api.addr_humanize(&staking_addr)? {
            return Ok(());
        }
    }
    if wallet.is_some() {
        return Ok(());
    }
    Err(ContractError::Unauthorized {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Staking {} => to_binary(&query_staking(deps)?),
        QueryMsg::Dao {} => to_binary(&query_dao(deps)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
    }
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES
        .may_load(deps.storage, &address)?
        .ok_or(StdError::GenericErr {
            msg: ContractError::NotFound {}.to_string(),
        })?;
    Ok(BalanceResponse { balance })
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub fn query_minter(deps: Deps) -> StdResult<Option<MinterResponse>> {
    let meta = TOKEN_INFO.load(deps.storage)?;
    let minter = match meta.mint {
        Some(m) => Some(MinterResponse {
            minter: m.minter.into(),
            cap: m.cap,
        }),
        None => None,
    };
    Ok(minter)
}

pub fn query_staking(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&STAKING_ADDR.load(deps.storage)?)
}

pub fn query_dao(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&DAO_ADDR.load(deps.storage)?)
}
