#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128,
};

use cw2::set_contract_version;
use cw20::{
    BalanceResponse, Cw20Coin, Cw20ReceiveMsg, Logo, MarketingInfoResponse, TokenInfoResponse,
};

use crate::enumerable::query_all_accounts;
use crate::error::ContractError;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    MinterData, TokenInfo, BALANCES, DAO_ADDR, MARKETING_INFO, STAKING_ADDR, TOKEN_INFO,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:govec";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
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

    // store token info
    // votes cannot be split, therefore decimals is 0
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: 0,
        total_supply,
        mint: msg.minter,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    // store marketing info
    if let Some(marketing) = msg.marketing {
        let marketing_info = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing
                .map(|addr| deps.api.addr_validate(addr.as_str()))
                .transpose()?,
            logo: None,
        };
        MARKETING_INFO.save(deps.storage, &marketing_info)?;
    }

    // store DAO contract addr
    DAO_ADDR.save(
        deps.storage,
        &deps.api.addr_canonicalize(info.sender.as_str())?,
    )?;

    if let Some(addr) = msg.staking_addr {
        STAKING_ADDR.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    }

    Ok(Response::default())
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
        ExecuteMsg::UpdateDaoAddr { new_addr } => execute_update_dao(deps, info, new_addr),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
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

/// Send can be used for the staking contract and other contracts in the future
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

    let m = new_mint.unwrap_or_default();
    let res = Response::new()
        .add_attribute("action", "update_minter_data")
        .add_attribute("new_minter", m.minter)
        .add_attribute("cap", m.cap.unwrap_or_default());

    Ok(res)
}

pub fn execute_update_dao(
    deps: DepsMut,
    info: MessageInfo,
    new_addr: String,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender)?;

    DAO_ADDR.save(deps.storage, &deps.api.addr_canonicalize(&new_addr)?)?;

    let res = Response::new()
        .add_attribute("action", "update_dao_address")
        .add_attribute("new_addr", new_addr);

    Ok(res)
}

pub fn execute_update_marketing(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>,
) -> Result<Response, ContractError> {
    match cw20_stake::contract::execute_update_marketing(
        deps,
        env,
        info,
        project,
        description,
        marketing,
    ) {
        Ok(res) => Ok(res),
        Err(res) => Err(ContractError::Cw20Stake(
            cw20_stake::ContractError::Cw20Error(res),
        )),
    }
}

pub fn execute_upload_logo(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    logo: Logo,
) -> Result<Response, ContractError> {
    match cw20_stake::contract::execute_upload_logo(deps, env, info, logo) {
        Ok(res) => Ok(res),
        Err(err) => Err(ContractError::Cw20Stake(
            cw20_stake::ContractError::Cw20Error(err),
        )),
    }
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
        QueryMsg::MarketingInfo {} => to_binary(&cw20_stake::contract::query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&cw20_stake::contract::query_download_logo(deps)?),
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

pub fn query_minter(deps: Deps) -> StdResult<Option<MinterData>> {
    let meta = TOKEN_INFO.load(deps.storage)?;
    Ok(meta.mint)
}

pub fn query_staking(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&STAKING_ADDR.load(deps.storage)?)
}

pub fn query_dao(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&DAO_ADDR.load(deps.storage)?)
}
