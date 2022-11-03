use crate::error::ContractError;
use crate::helpers::{
    create_mint_msg, ensure_enough_native_funds, ensure_is_dao, ensure_is_valid_migration_msg,
    ensure_is_valid_threshold, handle_govec_minted,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UnclaimedWalletList};
use crate::state::{
    ADDR_PREFIX, DAO, FEE, GOVEC_CLAIM_LIST, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID, TOTAL_CREATED,
};
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Expiration, DAY};
pub use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, CreateWalletMsg, Guardians,
    MigrationMsgError, ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction, RelayTxError,
    WalletAddr, WalletInfo, GOVEC_CLAIM_DURATION_DAY_MUL,
};
// use stake_cw20::msg::InstantiateMsg as StakingInstantiateMsg;
use std::ops::{Add, Mul};
use vectis_proxy::msg::{InstantiateMsg as ProxyInstantiateMsg, QueryMsg as ProxyQueryMsg};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
#[cfg(feature = "dao-chain")]
use {
    crate::helpers::ensure_has_govec, crate::state::GOVEC_MINTER, cosmwasm_std::from_binary,
    cw_utils::parse_reply_execute_data,
};
#[cfg(feature = "remote")]
use {crate::helpers::handle_govec_mint_failed, cosmwasm_std::StdError};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
// settings for pagination for unclaimed govec wallet list
const MAX_LIMIT: u32 = 1000;
const DEFAULT_LIMIT: u32 = 50;
/// Dao Chain govec contract reply
#[cfg(feature = "dao-chain")]
pub const GOVEC_REPLY_ID: u64 = u64::MIN;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;

    DAO.save(deps.storage, &admin_addr)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id)?;
    PROXY_MULTISIG_CODE_ID.save(deps.storage, &msg.proxy_multisig_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    FEE.save(deps.storage, &msg.wallet_fee)?;

    #[cfg(feature = "dao-chain")]
    if let Some(mint) = msg.govec_minter {
        GOVEC_MINTER.save(deps.storage, &deps.api.addr_canonicalize(&mint)?)?;
    };

    Ok(Response::new().add_attribute("Vectis Factory instantiated", env.contract.address))
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
            create_wallet(deps, info, env, create_wallet_msg)
        }
        ExecuteMsg::MigrateWallet {
            wallet_address,
            migration_msg,
        } => migrate_wallet(deps, info, wallet_address, migration_msg),
        ExecuteMsg::UpdateCodeId { ty, new_code_id } => update_code_id(deps, info, ty, new_code_id),
        ExecuteMsg::UpdateWalletFee { new_fee } => update_wallet_fee(deps, info, new_fee),
        ExecuteMsg::UpdateGovecAddr { addr } => update_govec_addr(deps, info, addr),
        ExecuteMsg::UpdateDao { addr } => update_dao_addr(deps, info, addr),
        ExecuteMsg::ClaimGovec {} => claim_govec_or_remove_from_list(deps, env, info),
        ExecuteMsg::GovecMinted {
            success,
            wallet_addr,
        } => govec_minted(deps, env, info, success, wallet_addr),
        ExecuteMsg::PurgeExpiredClaims { start_after, limit } => {
            purge_expired_claims(deps, env, start_after, limit)
        }
    }
}

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    #[cfg(feature = "dao-chain")]
    ensure_has_govec(deps.as_ref())?;

    let fee = FEE.load(deps.storage)?;
    let mut proxy_init_funds = create_wallet_msg.proxy_initial_funds.clone();
    let mut multisig_initial_funds = create_wallet_msg
        .guardians
        .guardians_multisig
        .clone()
        .unwrap_or_default()
        .multisig_initial_funds;

    // Ensure fixed multisig threshold is valid, if provided
    ensure_is_valid_threshold(&create_wallet_msg.guardians)?;
    ensure_enough_native_funds(
        &fee,
        &proxy_init_funds,
        &multisig_initial_funds,
        &info.funds,
    )?;

    // reply_id starts at 2 as 0 and 1 are occupied by consts
    if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(2) {
        proxy_init_funds.append(&mut multisig_initial_funds);
        // The wasm message containing the `wallet_proxy` instantiation message
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: PROXY_CODE_ID.load(deps.storage)?,
            msg: to_binary(&ProxyInstantiateMsg {
                multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                create_wallet_msg,
                code_id: PROXY_CODE_ID.load(deps.storage)?,
                addr_prefix: ADDR_PREFIX.load(deps.storage)?,
            })?,
            funds: proxy_init_funds.to_owned(),
            label: "Wallet-Proxy".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, next_id);
        let res = Response::new().add_submessage(msg);

        TOTAL_CREATED.save(deps.storage, &next_id)?;

        // Send native tokens to DAO to join the DAO
        if fee.amount != Uint128::zero() {
            let to_address = deps
                .api
                .addr_humanize(&DAO.load(deps.storage)?)?
                .to_string();

            // Direct transfer to DAO / remote_tunnel
            let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address,
                amount: vec![fee],
            });
            return Ok(res.add_message(bank_msg));
        }
        Ok(res)
    } else {
        Err(ContractError::OverFlow {})
    }
}

/// Migrates the instantiated `wallet_proxy` instance to a new code id
fn migrate_wallet(
    deps: DepsMut,
    info: MessageInfo,
    address: WalletAddr,
    migration_msg: ProxyMigrationTxMsg,
) -> Result<Response, ContractError> {
    let wallet_addr = match address {
        WalletAddr::Canonical(canonical_address) => deps.api.addr_humanize(&canonical_address)?,
        WalletAddr::Addr(human_address) => human_address,
    };

    let wallet_info: WalletInfo = deps
        .querier
        .query_wasm_smart(wallet_addr.clone(), &ProxyQueryMsg::Info {})?;

    // The migration call is either directly called by the user with `ProxyMigrationTxMsg::DirectMigrationMsg`
    // or relayed by the proxy relayer via `ProxyMigrationTxMsg::RelayTx`.
    //
    // Different safety checks are applied
    let tx_msg: CosmosMsg =
        ensure_is_valid_migration_msg(&deps, info, &wallet_info, &wallet_addr, migration_msg)?;

    // Further checks applied to ensure user has signed the correct relay msg / tx
    if let CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr,
        new_code_id,
        msg,
    }) = tx_msg.clone()
    {
        let msg: ProxyMigrateMsg = cosmwasm_std::from_slice(&msg)?;

        // Ensure user knows the latest supported proxy code id
        msg.ensure_is_supported_proxy_code_id(PROXY_CODE_ID.load(deps.storage)?)?;
        if new_code_id != PROXY_CODE_ID.load(deps.storage)? {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::MismatchProxyCodeId,
            ));
        }

        // Ensure migrating the corret wallet at given address
        if contract_addr != wallet_addr {
            return Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::InvalidWalletAddr,
            ));
        }
    } else {
        return Err(ContractError::InvalidMigrationMsg(
            MigrationMsgError::InvalidWasmMsg,
        ));
    }

    Ok(Response::new()
        .add_message(tx_msg)
        .add_attribute("action", "wallet migration"))
}

/// Updates the latest code id for the supported `wallet_proxy`
fn update_code_id(
    deps: DepsMut,
    info: MessageInfo,
    ty: CodeIdType,
    new_code_id: u64,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender.as_ref())?;
    match ty {
        CodeIdType::Proxy => {
            PROXY_CODE_ID.update(deps.storage, |c| {
                if c == new_code_id {
                    Err(ContractError::SameProxyCodeId {})
                } else {
                    Ok(new_code_id)
                }
            })?;
        }
        CodeIdType::Multisig => {
            PROXY_MULTISIG_CODE_ID.update(deps.storage, |c| {
                if c == new_code_id {
                    Err(ContractError::SameProxyMultisigCodeId {})
                } else {
                    Ok(new_code_id)
                }
            })?;
        }
    }
    Ok(Response::new()
        .add_attribute("config", "Code Id")
        .add_attribute("type", format!("{:?}", ty))
        .add_attribute("new Id", format!("{}", new_code_id)))
}

fn update_wallet_fee(
    deps: DepsMut,
    info: MessageInfo,
    new_fee: Coin,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender.as_str())?;
    FEE.save(deps.storage, &new_fee)?;
    Ok(Response::new()
        .add_attribute("config", "Wallet Fee")
        .add_attribute("New Fee", format!("{}", new_fee)))
}

#[cfg(feature = "dao-chain")]
fn update_govec_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender.as_str())?;
    GOVEC_MINTER.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "Govec Addr")
        .add_attribute("New Addr", addr))
}

#[cfg(feature = "remote")]
fn update_govec_addr(
    _deps: DepsMut,
    _info: MessageInfo,
    _addr: String,
) -> Result<Response, ContractError> {
    Err(ContractError::NotSupportedByChain {})
}

fn update_dao_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_dao(deps.as_ref(), info.sender.as_str())?;
    DAO.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "DAO")
        .add_attribute("New DAO", addr))
}

fn claim_govec_or_remove_from_list(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let claiming_user = deps.api.addr_canonicalize(info.sender.as_str())?.to_vec();
    if GOVEC_CLAIM_LIST
        .load(deps.storage, claiming_user.clone())?
        .is_expired(&env.block)
    {
        GOVEC_CLAIM_LIST.remove(deps.storage, claiming_user);
        Err(ContractError::ClaimExpired {})
    } else {
        let mint_msg = create_mint_msg(deps.as_ref(), info.sender.to_string())?;
        let res = Response::new()
            .add_submessage(mint_msg)
            .add_attribute("action", "Claim Govec Requested")
            .add_attribute("proxy_address", info.sender);
        Ok(res)
    }
}

#[cfg(feature = "remote")]
fn govec_minted(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    success: bool,
    wallet: String,
) -> Result<Response, ContractError> {
    let dao_minter = DAO.load(deps.storage)?;
    if info.sender != deps.api.addr_humanize(&dao_minter)?.to_string() {
        Err(ContractError::Unauthorized {})
    } else {
        if success {
            handle_govec_minted(deps, wallet)
        } else {
            handle_govec_mint_failed(deps, env, wallet)
        }
    }
}

#[cfg(feature = "dao-chain")]
fn govec_minted(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _sucess: bool,
    _wallet: String,
) -> Result<Response, ContractError> {
    Err(ContractError::NotSupportedByChain {})
}

fn purge_expired_claims(
    deps: DepsMut,
    env: Env,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Response, ContractError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            Some(Bound::exclusive(wallet_addr))
        }
        None => None,
    };

    let wallets: StdResult<Vec<(Vec<u8>, Expiration)>> = GOVEC_CLAIM_LIST
        .prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    for w in wallets? {
        if w.1.is_expired(&env.block) {
            GOVEC_CLAIM_LIST.remove(deps.storage, w.0)
        }
    }

    Ok(Response::default())
}

/// reply hooks handles replies from proxy wallet instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert

    let expected_id = TOTAL_CREATED
        .load(deps.storage)?
        .checked_add(1)
        .ok_or(ContractError::OverFlow {})?;

    if reply.id == expected_id {
        if let Ok(res) = parse_reply_instantiate_data(reply) {
            let wallet_addr: CanonicalAddr = deps.api.addr_canonicalize(&res.contract_address)?;
            let expiration = Expiration::AtTime(env.block.time)
                .add(DAY.mul(GOVEC_CLAIM_DURATION_DAY_MUL))
                .expect("error defining activate_at");

            GOVEC_CLAIM_LIST.save(deps.storage, wallet_addr.to_vec(), &expiration)?;

            let res = Response::new()
                .add_attribute("action", "Govec claim list updated")
                .add_attribute("proxy_address", res.contract_address);
            return Ok(res);
        } else {
            return Err(ContractError::ProxyInstantiationError {});
        }
    } else {
        #[cfg(feature = "dao-chain")]
        if reply.id == GOVEC_REPLY_ID {
            let res = parse_reply_execute_data(reply)?;
            if let Some(b) = res.data {
                let addr: String = from_binary(&b)?;
                return handle_govec_minted(deps, addr);
            } else {
                return Err(ContractError::InvalidReplyFromGovec {});
            }
        }

        return Err(ContractError::InvalidReplyId {});
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UnclaimedGovecWallets { start_after, limit } => {
            to_binary(&query_unclaim_wallet_list(deps, start_after, limit)?)
        }
        QueryMsg::ClaimExpiration { wallet } => {
            to_binary(&query_wallet_claim_expiration(deps, wallet)?)
        }
        QueryMsg::CodeId { ty } => to_binary(&query_code_id(deps, ty)?),
        QueryMsg::Fee {} => to_binary(&query_fee(deps)?),
        QueryMsg::GovecAddr {} => to_binary(&query_govec_addr(deps)?),
        QueryMsg::DaoAddr {} => to_binary(&query_dao_addr(deps)?),
        QueryMsg::TotalCreated {} => to_binary(&query_total(deps)?),
    }
}

/// Returns fees required for wallet creation
pub fn query_fee(deps: Deps) -> StdResult<Coin> {
    FEE.load(deps.storage)
}

/// Returns wallets created with limit
pub fn query_unclaim_wallet_list(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<UnclaimedWalletList> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let wallet_addr = deps.api.addr_canonicalize(&s)?.to_vec();
            Some(Bound::exclusive(wallet_addr))
        }
        None => None,
    };
    let wallets: StdResult<Vec<(Addr, Expiration)>> = GOVEC_CLAIM_LIST
        .prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| -> StdResult<(Addr, Expiration)> {
            let ww = w?;
            Ok((deps.api.addr_humanize(&CanonicalAddr::from(ww.0))?, ww.1))
        })
        .collect();

    Ok(UnclaimedWalletList { wallets: wallets? })
}
/// Returns wallets of user
pub fn query_wallet_claim_expiration(deps: Deps, wallet: String) -> StdResult<Option<Expiration>> {
    GOVEC_CLAIM_LIST.may_load(deps.storage, deps.api.addr_canonicalize(&wallet)?.to_vec())
}

/// Returns the current supported code Id:
/// - `wallet_proxy`
///  - `multisig` wallet user can use their own version, however we only support the cw3-fixed-multisig
pub fn query_code_id(deps: Deps, ty: CodeIdType) -> StdResult<u64> {
    let id = match ty {
        CodeIdType::Proxy => PROXY_CODE_ID.load(deps.storage)?,
        CodeIdType::Multisig => PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
    };
    Ok(id)
}
/// Returns govec token address
#[cfg(feature = "dao-chain")]
pub fn query_govec_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&GOVEC_MINTER.load(deps.storage)?)
}

#[cfg(feature = "remote")]
pub fn query_govec_addr(_deps: Deps) -> StdResult<Addr> {
    Err(StdError::GenericErr {
        msg: String::from("Not supported"),
    })
}

/// Returns DAO address
pub fn query_dao_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&DAO.load(deps.storage)?)
}

pub fn query_total(deps: Deps) -> StdResult<u64> {
    TOTAL_CREATED.load(deps.storage)
}
