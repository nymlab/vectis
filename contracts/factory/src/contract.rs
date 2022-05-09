use crate::error::ContractError;
use crate::helpers::{
    ensure_enough_native_funds, ensure_has_govec, ensure_is_admin, ensure_is_valid_migration_msg,
    ensure_is_valid_threshold,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use crate::state::{
    ADDR_PREFIX, ADMIN, FEE, GOVEC, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID, TOTAL_CREATED,
    WALLETS_OF,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use vectis_govec::msg::ExecuteMsg::Mint;
pub use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, CreateWalletMsg, Guardians,
    MigrationMsgError, ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction, RelayTxError,
    StakingOptions, WalletAddr, WalletInfo, WalletQueryPrefix,
};
// use stake_cw20::msg::InstantiateMsg as StakingInstantiateMsg;
use vectis_proxy::msg::{InstantiateMsg as ProxyInstantiateMsg, QueryMsg as ProxyQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// settings for pagination for wallet list
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;

    ADMIN.save(deps.storage, &admin_addr)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id)?;
    PROXY_MULTISIG_CODE_ID.save(deps.storage, &msg.proxy_multisig_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    FEE.save(deps.storage, &msg.wallet_fee)?;
    if let Some(addr) = msg.govec {
        GOVEC.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    }

    Ok(Response::new().add_attribute("method", "instantiate"))
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
        // TODO: update_code_id(new_code_id, enum::proxy / multisig, staking, govec)
        ExecuteMsg::UpdateCodeId { ty, new_code_id } => update_code_id(deps, info, ty, new_code_id),
        ExecuteMsg::UpdateWalletFee { new_fee } => update_wallet_fee(deps, info, new_fee),
        ExecuteMsg::UpdateGovecAddr { addr } => update_govec_addr(deps, info, addr),
        ExecuteMsg::UpdateAdmin { addr } => update_admin_addr(deps, info, addr),
    }
}

/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    ensure_has_govec(deps.as_ref())?;
    let fee = FEE.load(deps.storage)?;
    let proxy_init_funds = create_wallet_msg.proxy_initial_funds.clone();
    let multisig_initial_funds = create_wallet_msg
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

    if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(1) {
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
            funds: vec![proxy_init_funds, multisig_initial_funds].concat(),
            label: "Wallet-Proxy".into(),
        };
        let msg = SubMsg::reply_always(instantiate_msg, next_id);
        let res = Response::new().add_submessage(msg);

        // Send native tokens to admin to join the DAO
        TOTAL_CREATED.save(deps.storage, &next_id)?;

        if fee.amount != Uint128::zero() {
            let bank_msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&ADMIN.load(deps.storage)?)?
                    .to_string(),
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
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;
    match ty {
        CodeIdType::Proxy => {
            PROXY_CODE_ID.save(deps.storage, &new_code_id)?;
        }
        CodeIdType::Multisig => {
            PROXY_MULTISIG_CODE_ID.save(deps.storage, &new_code_id)?;
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
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    FEE.save(deps.storage, &new_fee)?;
    Ok(Response::new()
        .add_attribute("config", "Wallet Fee")
        .add_attribute("New Fee", format!("{}", new_fee)))
}

fn update_govec_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    GOVEC.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "Govec Addr")
        .add_attribute("New Addr", addr))
}

fn update_admin_addr(
    deps: DepsMut,
    info: MessageInfo,
    addr: String,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_str())?;
    ADMIN.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;
    Ok(Response::new()
        .add_attribute("config", "Admin")
        .add_attribute("New Admin", addr))
}

/// reply hooks handles 2 types of replies wrt to reply_id
/// - TOTAL_CREATED: proxy wallet instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, StdError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert

    let expected_id = TOTAL_CREATED.load(deps.storage)?;
    if reply.id == expected_id {
        let data = reply
            .result
            .into_result()
            .map_err(|err| StdError::generic_err(format!("Reply from proxy creation: {}", err)))?;
        let first_instantiate_event = data
            .events
            .iter()
            .find(|e| e.ty == "instantiate")
            .ok_or_else(|| StdError::generic_err("Reply: Unable to find reply event"))?;

        let str_addr = &first_instantiate_event.attributes[0].value;
        let wallet_addr: CanonicalAddr = deps.api.addr_canonicalize(str_addr)?;

        let user = data
            .events
            .iter()
            .find(|e| e.ty == "wasm")
            .ok_or_else(|| StdError::generic_err("Reply: Unable to find wasm event"))?
            .attributes
            .iter()
            .find(|k| k.key == "user")
            .ok_or_else(|| StdError::generic_err("Reply: Unable to find user attribute"))?;

        WALLETS_OF.save(
            deps.storage,
            (
                deps.api.addr_canonicalize(&user.value)?.to_vec(),
                wallet_addr.to_vec(),
            ),
            &(),
        )?;

        // Mint Govec Vote for the newly created proxy wallet
        let mint_msg: SubMsg = SubMsg::new(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&GOVEC.load(deps.storage)?)?
                .to_string(),
            msg: to_binary(&Mint {
                new_wallet: str_addr.into(),
            })?,
            funds: vec![],
        });

        let res = Response::new()
            .add_submessage(mint_msg)
            .add_attribute("action", "Wallet Proxy Stored")
            .add_attribute("proxy_address", str_addr);
        Ok(res)
    } else {
        Err(StdError::GenericErr {
            msg: ContractError::InvalidReplyId {}.to_string(),
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Wallets { start_after, limit } => {
            to_binary(&query_wallet_list(deps, start_after, limit)?)
        }
        QueryMsg::WalletsOf {
            user,
            start_after,
            limit,
        } => to_binary(&query_wallets_of(deps, user, start_after, limit)?),
        QueryMsg::CodeId { ty } => to_binary(&query_code_id(deps, ty)?),
        QueryMsg::Fee {} => to_binary(&query_fee(deps)?),
        QueryMsg::GovecAddr {} => to_binary(&query_govec_addr(deps)?),
        QueryMsg::AdminAddr {} => to_binary(&query_admin_addr(deps)?),
    }
}

/// Returns fees required for wallet creation
pub fn query_fee(deps: Deps) -> StdResult<Coin> {
    FEE.load(deps.storage)
}

/// Returns wallets created with limit
pub fn query_wallet_list(
    deps: Deps,
    start_after: Option<WalletQueryPrefix>,
    limit: Option<u32>,
) -> StdResult<WalletListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let user_addr = deps.api.addr_canonicalize(s.user_addr.as_str())?.to_vec();
            let wallet_addr = deps.api.addr_canonicalize(s.wallet_addr.as_str())?.to_vec();
            Some(Bound::exclusive((user_addr, wallet_addr)))
        }
        None => None,
    };
    let wallets: Result<Vec<_>, _> = WALLETS_OF
        .sub_prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| -> StdResult<Addr> { deps.api.addr_humanize(&CanonicalAddr::from(w?.0 .1)) })
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
}
/// Returns wallets of user
pub fn query_wallets_of(
    deps: Deps,
    user: String,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<WalletListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = match start_after {
        Some(s) => {
            let addr = deps.api.addr_canonicalize(&s)?;
            Some(Bound::ExclusiveRaw(addr.into()))
        }
        None => None,
    };
    let user_addr = deps.api.addr_validate(&user)?;
    let user_addr = deps.api.addr_canonicalize(user_addr.as_str())?;

    let wallets: Result<Vec<_>, _> = WALLETS_OF
        .prefix(user_addr.to_vec())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key?.0)))
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
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
pub fn query_govec_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&GOVEC.load(deps.storage)?)
}

/// Returns admin address
pub fn query_admin_addr(deps: Deps) -> StdResult<Addr> {
    deps.api.addr_humanize(&ADMIN.load(deps.storage)?)
}
