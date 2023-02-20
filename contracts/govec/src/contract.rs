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
    TokenInfo, BALANCES, DAO_ADDR, FACTORY, ITEMS, MARKETING_INFO, MINT_AMOUNT, MINT_CAP,
    STAKING_ADDR, TOKEN_INFO,
};
use cw20_stake::{
    contract::{
        execute_update_marketing, execute_upload_logo, query_download_logo, query_marketing_info,
    },
    ContractError::Cw20Error,
};
use vectis_wallet::{MintResponse, UpdateAddrReq};

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

    if let Some(limit) = msg.mint_cap {
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

    MINT_AMOUNT.save(deps.storage, &msg.mint_amount)?;

    if let Some(addr) = msg.staking_addr {
        STAKING_ADDR.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    }
    if let Some(addr) = msg.factory {
        FACTORY.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    }
    if let Some(amount) = msg.mint_cap {
        MINT_CAP.save(deps.storage, &amount)?;
    }

    // ensure that the DAO can recieve Govec if it does not have initial balance
    if BALANCES.may_load(deps.storage, &info.sender)?.is_none() {
        BALANCES.save(deps.storage, &info.sender, &Uint128::zero())?;
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
        ExecuteMsg::Transfer {
            recipient,
            amount,
            relayed_from,
        } => execute_transfer(deps, info, recipient, amount, relayed_from),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, info, owner, recipient, amount),
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Exit { relayed_from } => execute_exit(deps, env, info, relayed_from),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
            relayed_from,
        } => execute_send(deps, env, info, contract, amount, msg, relayed_from),
        ExecuteMsg::Mint { new_wallet } => execute_mint(deps, env, info, new_wallet),
        ExecuteMsg::UpdateConfigAddr { new_addr } => {
            execute_update_config_addr(deps, info, new_addr)
        }
        ExecuteMsg::UpdateMintAmount { new_amount } => {
            execute_update_mint_amount(deps, info, new_amount)
        }
        ExecuteMsg::UpdateMintCap { new_mint_cap } => {
            execute_update_mint_cap(deps, info, new_mint_cap)
        }
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => govec_execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => govec_execute_upload_logo(deps, env, info, logo),
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
    relayed_from: Option<String>,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let from = match relayed_from {
        Some(remote) => {
            ensure_is_dao_tunnel(deps.as_ref(), info.sender)?;
            Addr::unchecked(remote)
        }
        None => info.sender,
    };

    // TODO: catch error that is the wrong prefix
    // let rcpt_addr = deps.api.addr_validate(&recipient)?;
    let rcpt_addr = Addr::unchecked(recipient);
    ensure_is_wallet_or_authorised(deps.as_ref(), &rcpt_addr)?;

    BALANCES.update(
        deps.storage,
        &from,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        },
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", from)
        .add_attribute("to", rcpt_addr)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_transfer_from(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let dao = DAO_ADDR.load(deps.storage)?;
    let pre_proposal = ITEMS
        .query(
            &deps.querier,
            deps.api.addr_humanize(&dao)?,
            "pre-proposal".into(),
        )?
        .ok_or(ContractError::NotFound {})?;
    if info.sender != pre_proposal {
        return Err(ContractError::Unauthorized {});
    }

    let rcpt_addr = Addr::unchecked(recipient);
    ensure_is_wallet_or_authorised(deps.as_ref(), &rcpt_addr)?;

    BALANCES.update(
        deps.storage,
        &Addr::unchecked(owner),
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        },
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer_from")
        .add_attribute("from", info.sender)
        .add_attribute("to", rcpt_addr)
        .add_attribute("amount", amount);
    Ok(res)
}

/// Burning of Govec
/// must be done by dao
pub fn execute_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), &info.sender)?;

    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    // lower balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    // reduce total_supply
    TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
        info.total_supply = info.total_supply.checked_sub(amount)?;
        Ok(info)
    })?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount);

    Ok(res)
}

/// Exits the DAO
/// remove the requester from the ledger
/// any balance in the requester account will be sent to the dao
pub fn execute_exit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    relayed_from: Option<String>,
) -> Result<Response, ContractError> {
    let from = match relayed_from {
        Some(remote) => {
            ensure_is_dao_tunnel(deps.as_ref(), info.sender)?;
            Addr::unchecked(remote)
        }
        None => info.sender,
    };
    // Ensure only have voting power of exactly 1
    let remaining_balance = query_balance(deps.as_ref(), from.to_string())?.balance;

    let dao = deps.api.addr_humanize(&DAO_ADDR.load(deps.storage)?)?;

    BALANCES.update(
        deps.storage,
        &dao,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(remaining_balance)?)
        },
    )?;

    // remove key from the map as they exit the DAO
    BALANCES.remove(deps.storage, &from);

    Ok(Response::new()
        .add_attribute("action", "exit")
        .add_attribute("addr", from))
}

pub enum Role {
    Factory,
    DaoTunnel,
    Dao,
}

fn ensure_is_minter(deps: Deps, sender: Addr) -> Result<Role, ContractError> {
    let d = DAO_ADDR.load(deps.storage)?;
    let t = ITEMS
        .query(
            &deps.querier,
            deps.api.addr_humanize(&d)?,
            "dao-tunnel".into(),
        )?
        .ok_or_else(|| {
            println!("cannot get query");
            ContractError::NotFound {}
        })?;
    let f = FACTORY.load(deps.storage)?;
    if sender == t {
        Ok(Role::DaoTunnel)
    } else if sender == deps.api.addr_humanize(&f)? {
        Ok(Role::Factory)
    } else if sender == deps.api.addr_humanize(&d)? {
        Ok(Role::Dao)
    } else {
        Err(ContractError::Unauthorized {})
    }
}

pub fn execute_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_wallet: String,
) -> Result<Response, ContractError> {
    // update supply and enforce cap
    let caller = ensure_is_minter(deps.as_ref(), info.sender)?;
    let mint_amount = MINT_AMOUNT.load(deps.storage)?;

    let mut config = TOKEN_INFO.load(deps.storage)?;
    config.total_supply += mint_amount;
    if let Some(limit) = MINT_CAP.may_load(deps.storage)? {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    let rcpt_addr = match caller {
        Role::Factory => deps.api.addr_validate(&new_wallet)?,
        // We do validate remote wallet address with Bech32 as prefix will be different
        // Validation is done on the remote-tunnel channel
        Role::DaoTunnel => Addr::unchecked(new_wallet.clone()),
        Role::Dao => Addr::unchecked(new_wallet.clone()),
    };

    // add amount to recipient balance
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default() + mint_amount)
        },
    )?;

    let res = Response::new()
        .set_data(to_binary(&new_wallet)?)
        .add_attribute("action", "mint")
        .add_attribute("to", new_wallet)
        .add_attribute("amount", mint_amount.to_string());
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
    relayed_from: Option<String>,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }
    let from = match relayed_from {
        Some(remote) => {
            ensure_is_dao_tunnel(deps.as_ref(), info.sender)?;
            Addr::unchecked(remote)
        }
        None => info.sender,
    };

    let contract = deps.api.addr_validate(&contract)?;
    ensure_is_wallet_or_authorised(deps.as_ref(), &contract)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &from,
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
        .add_attribute("from", &from)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(
            Cw20ReceiveMsg {
                sender: from.into(),
                amount,
                msg,
            }
            .into_cosmos_msg(contract)?,
        );
    Ok(res)
}

pub fn execute_update_mint_amount(
    deps: DepsMut,
    info: MessageInfo,
    new_amount: Uint128,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), &info.sender)?;
    MINT_AMOUNT.save(deps.storage, &new_amount)?;
    Ok(Response::new()
        .add_attribute("action", "update_mint_amount")
        .add_attribute("amount", new_amount.to_string()))
}

pub fn execute_update_mint_cap(
    deps: DepsMut,
    info: MessageInfo,
    new_mint: Option<Uint128>,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), &info.sender)?;

    let res = match new_mint {
        Some(cap) => {
            MINT_CAP.save(deps.storage, &cap)?;
            Response::new()
                .add_attribute("action", "update_mint_cap")
                .add_attribute("cap", cap)
        }
        None => {
            MINT_CAP.remove(deps.storage);
            Response::new().add_attribute("action", "removed_mint_cap")
        }
    };

    Ok(res)
}

pub fn execute_update_config_addr(
    deps: DepsMut,
    info: MessageInfo,
    new_addr: UpdateAddrReq,
) -> Result<Response, ContractError> {
    let dao = ensure_is_dao(deps.as_ref(), &info.sender)?;
    let res = match new_addr {
        UpdateAddrReq::Dao(addr) => {
            let new_dao = deps.api.addr_validate(&addr)?;
            DAO_ADDR.save(deps.storage, &deps.api.addr_canonicalize(new_dao.as_str())?)?;
            // transfer all balance from existing DAO to the new DAO
            let existing_dao_balance = BALANCES.may_load(deps.storage, dao)?;
            let new_dao_balance = BALANCES.may_load(deps.storage, &new_dao)?;

            if let Some(amount) = existing_dao_balance {
                if new_dao_balance.is_some() {
                    BALANCES.update(
                        deps.storage,
                        &new_dao,
                        |balance: Option<Uint128>| -> StdResult<_> {
                            Ok(balance.unwrap_or_default().checked_add(amount)?)
                        },
                    )?;
                } else {
                    BALANCES.save(deps.storage, &new_dao, &amount)?;
                }

                BALANCES.save(deps.storage, dao, &Uint128::zero())?;
            };

            Response::new()
                .add_attribute("action", "update_dao")
                .add_attribute("new_addr", addr)
        }
        UpdateAddrReq::Factory(addr) => {
            FACTORY.save(
                deps.storage,
                &deps
                    .api
                    .addr_canonicalize(deps.api.addr_validate(&addr)?.as_str())?,
            )?;
            Response::new()
                .add_attribute("action", "update_factory")
                .add_attribute("new_addr", addr)
        }
        UpdateAddrReq::Staking(addr) => {
            STAKING_ADDR.save(
                deps.storage,
                &deps
                    .api
                    .addr_canonicalize(deps.api.addr_validate(&addr)?.as_str())?,
            )?;
            Response::new()
                .add_attribute("action", "update_staking")
                .add_attribute("new_addr", addr)
        }
    };

    Ok(res)
}

pub fn govec_execute_update_marketing(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>,
) -> Result<Response, ContractError> {
    match execute_update_marketing(deps, env, info, project, description, marketing) {
        Ok(res) => Ok(res),
        Err(res) => Err(ContractError::Cw20Stake(Cw20Error(res))),
    }
}

pub fn govec_execute_upload_logo(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    logo: Logo,
) -> Result<Response, ContractError> {
    match execute_upload_logo(deps, env, info, logo) {
        Ok(res) => Ok(res),
        Err(err) => Err(ContractError::Cw20Stake(Cw20Error(err))),
    }
}

fn ensure_is_dao_tunnel(deps: Deps, sender: Addr) -> Result<Addr, ContractError> {
    let dao = DAO_ADDR.load(deps.storage)?;
    let dao_tunnel = ITEMS
        .query(
            &deps.querier,
            deps.api.addr_humanize(&dao)?,
            "dao-tunnel".into(),
        )?
        .ok_or(ContractError::NotFound {})?;
    if dao_tunnel != sender {
        return Err(ContractError::Unauthorized {});
    }
    Ok(sender)
}

fn ensure_is_dao<'a>(deps: Deps, sender: &'a Addr) -> Result<&'a Addr, ContractError> {
    let dao = DAO_ADDR.load(deps.storage)?;
    if dao != deps.api.addr_canonicalize(sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }
    Ok(sender)
}

// transfer is ok if the recipient is a wallet (is in the balances ledger)
// or, it is staking / pre_preposal contracts
fn ensure_is_wallet_or_authorised(deps: Deps, contract: &Addr) -> Result<(), ContractError> {
    let dao = DAO_ADDR.load(deps.storage)?;
    let pre_proposal = ITEMS.query(
        &deps.querier,
        deps.api.addr_humanize(&dao)?,
        "pre-proposal".into(),
    )?;
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
    if let Some(prep) = pre_proposal {
        if *contract == prep {
            return Ok(());
        }
    }
    Err(ContractError::Unauthorized {})
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::Joined { address } => to_binary(&query_balance_joined(deps, address)?),
        QueryMsg::MintAmount {} => to_binary(&query_mint_amount(deps)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minters {} => to_binary(&query_minter(deps)?),
        QueryMsg::Staking {} => to_binary(&query_staking(deps)?),
        QueryMsg::Dao {} => to_binary(&query_dao(deps)?),
        QueryMsg::DaoTunnel {} => to_binary(&query_dao_tunnel(deps)?),
        QueryMsg::Factory {} => to_binary(&query_factory(deps)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
        QueryMsg::TokenContract {} => to_binary(&query_contract_addr(env)),
    }
}
pub fn query_contract_addr(env: Env) -> Addr {
    env.contract.address
}

pub fn query_balance_joined(deps: Deps, address: String) -> StdResult<Option<BalanceResponse>> {
    Ok(BALANCES
        .load(deps.storage, &Addr::unchecked(address))
        .map(|balance| BalanceResponse { balance })
        .ok())
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let balance = query_balance_joined(deps, address).unwrap_or(None);
    Ok(balance.unwrap_or(BalanceResponse {
        balance: Uint128::new(0),
    }))
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

pub fn query_mint_amount(deps: Deps) -> StdResult<Uint128> {
    MINT_AMOUNT.load(deps.storage)
}

pub fn query_minter(deps: Deps) -> StdResult<MintResponse> {
    let dao = DAO_ADDR.load(deps.storage)?;
    let mut v = Vec::new();
    v.push(deps.api.addr_humanize(&dao)?.to_string());

    let d = ITEMS.query(
        &deps.querier,
        deps.api.addr_humanize(&dao)?,
        "dao-tunnel".into(),
    )?;
    let f = FACTORY.may_load(deps.storage)?;
    let cap = MINT_CAP.may_load(deps.storage)?;
    if let Some(daot) = d {
        v.push(daot);
    }
    if let Some(factory) = f {
        v.push(deps.api.addr_humanize(&factory)?.to_string());
    }

    Ok(MintResponse {
        minters: Some(v),
        cap,
    })
}

pub fn query_staking(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&STAKING_ADDR.load(deps.storage)?)
}

pub fn query_dao(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&DAO_ADDR.load(deps.storage)?)
}

pub fn query_dao_tunnel(deps: Deps) -> StdResult<Addr> {
    let dao = DAO_ADDR.load(deps.storage)?;
    let d = ITEMS
        .query(
            &deps.querier,
            deps.api.addr_humanize(&dao)?,
            "dao-tunnel".into(),
        )?
        .ok_or(StdError::NotFound {
            kind: "DAO tunnel not found".to_string(),
        })?;
    deps.api.addr_validate(&d)
}

pub fn query_factory(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&FACTORY.load(deps.storage)?)
}
