use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use crate::state::{
    ADDR_PREFIX, ADMIN, FEE, GOVEC, GOVEC_CODE_ID, PROXY_CODE_ID, PROXY_MULTISIG_CODE_ID, STAKE,
    STAKING_CODE_ID, TOTAL_CREATED, WALLETS_OF,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, Event,
    MessageInfo, Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw1::CanExecuteResponse;
use cw2::set_contract_version;
use cw20::Cw20Coin;
use cw_storage_plus::Bound;
use govec::msg::{
    ExecuteMsg::Mint, InstantiateMsg as GovecInstantiateMsg, MinterResponse, StakingOptions,
};

pub use sc_wallet::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, CreateWalletMsg, Guardians,
    MigrationMsgError, ProxyMigrateMsg, ProxyMigrationTxMsg, RelayTransaction, RelayTxError,
    WalletAddr, WalletInfo,
};
// use stake_cw20::msg::InstantiateMsg as StakingInstantiateMsg;
use wallet_proxy::msg::{InstantiateMsg as ProxyInstantiateMsg, QueryMsg as ProxyQueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:smart-contract-wallet-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GOV_REPLY_ID: u64 = u64::MIN;

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
    STAKING_CODE_ID.save(deps.storage, &msg.staking_code_id)?;
    GOVEC_CODE_ID.save(deps.storage, &msg.govec_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    FEE.save(deps.storage, &msg.wallet_fee)?;

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
        ExecuteMsg::CreateGovernance {
            staking_options,
            initial_balances,
        } => create_governance(deps, info, env, staking_options, initial_balances),
    }
}

/// Creates the governance structure for the DAO
/// This instantiates the govec token contract,
/// which instantiates the the staking contract
/// Admins for these contracts is the DAO
/// Minter of the govec token contract is this factory
fn create_governance(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    staking: Option<StakingOptions>,
    initial_balances: Vec<Cw20Coin>,
) -> Result<Response, ContractError> {
    ensure_is_admin(deps.as_ref(), info.sender.as_ref())?;
    let dao = ADMIN.load(deps.storage)?;
    let govec_inst_msg = GovecInstantiateMsg {
        name: "Vectis Governance Token Contract".into(),
        symbol: "VEC".into(),
        initial_balances,
        staking,
        mint: Some(MinterResponse {
            minter: env.contract.address.to_string(),
            cap: None,
        }),
        dao: dao.clone(),
    };

    Ok(Response::new().add_submessage(SubMsg::reply_always(
        WasmMsg::Instantiate {
            admin: Some(deps.api.addr_humanize(&dao)?.to_string()),
            code_id: GOVEC_CODE_ID.load(deps.storage)?,
            msg: to_binary(&govec_inst_msg)?,
            funds: vec![],
            label: "Governance Token".into(),
        },
        GOV_REPLY_ID,
    )))
}

/// Ensure user has sent in enough to cover the fee and the initial proxy balance
fn ensure_enough_native_funds(
    fee: Coin,
    proxy_initial_fund: Vec<Coin>,
    sent_fund: Vec<Coin>,
) -> Result<(), ContractError> {
    let init_native_fund = proxy_initial_fund.iter().fold(Uint128::zero(), |acc, c| {
        if c.denom == fee.denom {
            acc + c.amount
        } else {
            acc
        }
    });

    let total_native_fund = fee.amount + init_native_fund;

    let total_sent = sent_fund.iter().fold(Uint128::zero(), |acc, c| {
        if c.denom == fee.denom {
            acc + c.amount
        } else {
            acc
        }
    });

    if total_native_fund == total_sent {
        Ok(())
    } else {
        Err(ContractError::InvalidNativeFund(
            total_native_fund,
            total_sent,
        ))
    }
}
/// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
fn create_wallet(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    create_wallet_msg: CreateWalletMsg,
) -> Result<Response, ContractError> {
    // Ensure fixed multisig threshold is valid, if provided
    let fee = FEE.load(deps.storage)?;
    ensure_is_valid_threshold(&create_wallet_msg.guardians)?;
    ensure_enough_native_funds(
        fee.clone(),
        create_wallet_msg.proxy_initial_funds.clone(),
        info.funds,
    )?;

    if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(1) {
        // The wasm message containing the `wallet_proxy` instantiation message
        let instantiate_msg = WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: PROXY_CODE_ID.load(deps.storage)?,
            msg: to_binary(&ProxyInstantiateMsg {
                multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                create_wallet_msg: create_wallet_msg.clone(),
                code_id: PROXY_CODE_ID.load(deps.storage)?,
                addr_prefix: ADDR_PREFIX.load(deps.storage)?,
            })?,
            funds: create_wallet_msg.proxy_initial_funds,
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

// Perform security checks to ensure migration message is valid
fn ensure_is_valid_migration_msg(
    deps: &DepsMut,
    info: MessageInfo,
    wallet_info: &WalletInfo,
    wallet_addr: &Addr,
    migration_msg: ProxyMigrationTxMsg,
) -> Result<CosmosMsg, ContractError> {
    let tx_msg: CosmosMsg = match migration_msg {
        ProxyMigrationTxMsg::RelayTx(tx) => {
            let can_execute_relay: CanExecuteResponse = deps.querier.query_wasm_smart(
                wallet_addr.clone(),
                &ProxyQueryMsg::CanExecuteRelay {
                    sender: info.sender.to_string(),
                },
            )?;

            // Ensure caller is a wallet relayer
            if !can_execute_relay.can_execute {
                return Err(ContractError::Unauthorized {});
            } else {
                // Ensure Signer of relayed message is the wallet user
                if wallet_info.user_addr
                    != pub_key_to_address(
                        deps,
                        &ADDR_PREFIX.load(deps.storage)?,
                        &tx.user_pubkey.0,
                    )?
                {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::IsNotUser {},
                    ));
                };

                // Ensure none of relayed message is the expected next wallet nonce
                if wallet_info.nonce != tx.nonce {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::NoncesAreNotEqual {},
                    ));
                };

                // Verify signature
                if !query_verify_cosmos(deps, &tx)? {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::SignatureVerificationError {},
                    ));
                };

                cosmwasm_std::from_slice(tx.message.0.as_slice())?
            }
        }
        ProxyMigrationTxMsg::DirectMigrationMsg(msg) => {
            // Ensure caller is the wallet user
            if wallet_info.user_addr != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            cosmwasm_std::from_slice(&msg)?
        }
    };
    Ok(tx_msg)
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
        CodeIdType::Govec => {
            GOVEC_CODE_ID.save(deps.storage, &new_code_id)?;
        }
        CodeIdType::Staking => {
            STAKING_CODE_ID.save(deps.storage, &new_code_id)?;
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

/// reply hooks handles 2 types of replies wrt to reply_id
/// - GOV_REPLY_ID: govec token contract instantiation reply in create_governance
/// - TOTAL_CREATED: proxy wallet instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, StdError> {
    // NOTE: Error returned in `reply` is equivalent to contract error, all states revert,
    // specifically, the TOTAL_CREATED incremented in `create_wallet` will revert

    // Handles create_governance reply.
    // Specifically, we save the Govec token contract and staking contract addresses
    if reply.id == GOV_REPLY_ID {
        let data = reply.result.into_result().map_err(StdError::generic_err)?;
        let instantiate_events: Vec<&Event> = data
            .events
            .iter()
            .filter(|e| e.ty == "instantiate")
            .collect();

        // There are 2 instantiations events
        // 1. Govec contract
        // 2. Stake-cw20 contract
        if instantiate_events.len() != 2 {
            return Err(StdError::generic_err(
                "Invalid reply events for create_governance",
            ));
        }
        GOVEC.save(
            deps.storage,
            &deps
                .api
                .addr_canonicalize(&instantiate_events[0].attributes[0].value)?,
        )?;
        STAKE.save(
            deps.storage,
            &deps
                .api
                .addr_canonicalize(&instantiate_events[1].attributes[0].value)?,
        )?;

        let res = Response::new()
            .add_attribute("action", "Govec Deployed")
            .add_attribute("Govec", &instantiate_events[0].attributes[0].value);
        Ok(res)
    } else {
        let expected_id = TOTAL_CREATED.load(deps.storage)?;
        if reply.id == expected_id {
            let data = reply.result.into_result().map_err(StdError::generic_err)?;
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
                    deps.api.addr_canonicalize(&user.value)?.as_slice(),
                    wallet_addr.as_slice(),
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
        QueryMsg::ProxyCodeId {} => to_binary(&query_proxy_code_id(deps)?),
        QueryMsg::MultisigCodeId {} => to_binary(&query_multisig_code_id(deps)?),
        QueryMsg::Fee {} => to_binary(&query_fee(deps)?),
    }
}

/// Returns fees required for wallet creation
pub fn query_fee(deps: Deps) -> StdResult<Coin> {
    FEE.load(deps.storage)
}

/// Returns wallets created with limit
pub fn query_wallet_list(
    deps: Deps,
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
    let wallets: Result<Vec<_>, _> = WALLETS_OF
        .sub_prefix(())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|w| deps.api.addr_humanize(&CanonicalAddr::from(w?.0 .1)))
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
        .prefix(user_addr.as_slice())
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|key| deps.api.addr_humanize(&CanonicalAddr::from(key?.0)))
        .collect();

    Ok(WalletListResponse { wallets: wallets? })
}

/// Returns the current supported `wallet_proxy` code id
pub fn query_proxy_code_id(deps: Deps) -> StdResult<u64> {
    let id = PROXY_CODE_ID.load(deps.storage)?;
    Ok(id)
}

/// Returns the current default `multisig` code id for `wallet_proxy`
/// wallet user can use their own version, however we only support the cw3-fixed-multisig
/// `instantiateMsg` for the time being
pub fn query_multisig_code_id(deps: Deps) -> StdResult<u64> {
    let id = PROXY_MULTISIG_CODE_ID.load(deps.storage)?;
    Ok(id)
}

/// Ensures provided addr is the state stored ADMIN
fn ensure_is_admin(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller == admin {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

/// Ensures provided fixed multisig threshold is valid.
fn ensure_is_valid_threshold(guardians: &Guardians) -> Result<(), ContractError> {
    match &guardians.guardians_multisig {
        Some(g) if g.threshold_absolute_count == 0 => {
            Err(ContractError::ThresholdShouldBeGreaterThenZero {})
        }
        Some(g) if g.threshold_absolute_count > guardians.addresses.len() as u64 => {
            Err(ContractError::ThresholdShouldBeLessThenGuardiansCount {})
        }
        _ => Ok(()),
    }
}
