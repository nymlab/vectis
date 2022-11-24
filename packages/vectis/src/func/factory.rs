use std::ops::{Add, Mul};

use cosmwasm_std::{
    to_binary, Addr, BankMsg, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Reply, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw_storage_plus::Bound;
use cw_utils::{parse_reply_instantiate_data, Expiration, DAY};

use cw1::CanExecuteResponse;

use crate::{factory_state::*, GOVEC_CLAIM_DURATION_DAY_MUL};
use crate::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, FactoryError as ContractError, Guardians,
    ProxyMigrationTxMsg, ProxyQueryMsg, RelayTxError,
    WalletFactoryInstantiateMsg as InstantiateMsg, WalletInfo,
};

// settings for pagination for unclaimed govec wallet list
const MAX_LIMIT: u32 = 1000;
const DEFAULT_LIMIT: u32 = 50;

pub fn factory_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;

    DAO.save(deps.storage, &admin_addr)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id)?;
    PROXY_MULTISIG_CODE_ID.save(deps.storage, &msg.proxy_multisig_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    WALLET_FEE.save(deps.storage, &msg.wallet_fee)?;
    CLAIM_FEE.save(deps.storage, &msg.claim_fee)?;

    Ok(Response::new().add_attribute("Vectis Factory instantiated", env.contract.address))
}

pub mod factory_execute {
    use super::*;

    use crate::{
        CreateWalletMsg, MigrationMsgError, ProxyInstantiateMsg, ProxyMigrateMsg, UpdateFeeReq,
        WalletAddr,
    };

    /// Creates a SCW by instantiating an instance of the `wallet_proxy` contract
    pub fn create_wallet(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        create_wallet_msg: CreateWalletMsg,
    ) -> Result<Response, ContractError> {
        let fee = WALLET_FEE.load(deps.storage)?;
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

        // reply_id starts at 1  as 0 occupied by const GOVEC_REPLY_ID
        if let Some(next_id) = TOTAL_CREATED.load(deps.storage)?.checked_add(1) {
            proxy_init_funds.append(&mut multisig_initial_funds);
            let funds = if proxy_init_funds.is_empty() {
                vec![]
            } else {
                proxy_init_funds
            };

            // The wasm message containing the `wallet_proxy` instantiation message
            let instantiate_msg = WasmMsg::Instantiate {
                admin: Some(env.contract.address.to_string()),
                code_id: PROXY_CODE_ID.load(deps.storage)?,
                msg: to_binary(&ProxyInstantiateMsg {
                    multisig_code_id: PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
                    create_wallet_msg,
                    code_id: PROXY_CODE_ID.load(deps.storage)?,
                })?,
                funds,
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
    pub fn migrate_wallet(
        deps: DepsMut,
        info: MessageInfo,
        address: WalletAddr,
        migration_msg: ProxyMigrationTxMsg,
    ) -> Result<Response, ContractError> {
        let wallet_addr = match address {
            WalletAddr::Canonical(canonical_address) => {
                deps.api.addr_humanize(&canonical_address)?
            }
            WalletAddr::Addr(human_address) => human_address,
        };

        let wallet_info: WalletInfo = deps
            .querier
            .query_wasm_smart(wallet_addr.clone(), &ProxyQueryMsg::Info {})?;

        // The migration call is either directly called by the controller with `ProxyMigrationTxMsg::DirectMigrationMsg`
        // or relayed by the proxy relayer via `ProxyMigrationTxMsg::RelayTx`.
        //
        // Different safety checks are applied
        let tx_msg: CosmosMsg =
            ensure_is_valid_migration_msg(&deps, info, &wallet_info, &wallet_addr, migration_msg)?;

        // Further checks applied to ensure controller has signed the correct relay msg / tx
        if let CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr,
            new_code_id,
            msg,
        }) = tx_msg.clone()
        {
            let msg: ProxyMigrateMsg = cosmwasm_std::from_slice(&msg)?;

            // Ensure controller knows the latest supported proxy code id
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
    pub fn update_code_id(
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

    pub fn update_config_fee(
        deps: DepsMut,
        info: MessageInfo,
        new_fee: UpdateFeeReq,
    ) -> Result<Response, ContractError> {
        ensure_is_dao(deps.as_ref(), info.sender.as_str())?;

        let res = match new_fee {
            UpdateFeeReq::Claim(fee) => {
                CLAIM_FEE.save(deps.storage, &fee)?;
                Response::new()
                    .add_attribute("config", "Claim Fee")
                    .add_attribute("New Fee", format!("{}", fee))
            }
            UpdateFeeReq::Wallet(fee) => {
                WALLET_FEE.save(deps.storage, &fee)?;
                Response::new()
                    .add_attribute("config", "Wallet Fee")
                    .add_attribute("New Fee", format!("{}", fee))
            }
        };

        Ok(res)
    }

    pub fn update_dao_addr(
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

    pub fn purge_expired_claims(
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
}

pub mod factory_queries {
    use super::*;

    use crate::{FeesResponse, UnclaimedWalletList};

    /// Returns wallets with Govec to claim with limit
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

    /// Returns wallets of controller
    pub fn query_wallet_claim_expiration(
        deps: Deps,
        wallet: String,
    ) -> StdResult<Option<Expiration>> {
        GOVEC_CLAIM_LIST.may_load(deps.storage, deps.api.addr_canonicalize(&wallet)?.to_vec())
    }

    /// Returns the current supported code Id:
    /// - `wallet_proxy`
    ///  - `multisig` wallet controller can use their own version, however we only support the cw3-fixed-multisig
    pub fn query_code_id(deps: Deps, ty: CodeIdType) -> StdResult<u64> {
        let id = match ty {
            CodeIdType::Proxy => PROXY_CODE_ID.load(deps.storage)?,
            CodeIdType::Multisig => PROXY_MULTISIG_CODE_ID.load(deps.storage)?,
        };
        Ok(id)
    }

    /// Returns fees required for wallet creation and claim govec
    pub fn query_fees(deps: Deps) -> StdResult<FeesResponse> {
        Ok(FeesResponse {
            wallet_fee: WALLET_FEE.load(deps.storage)?,
            claim_fee: CLAIM_FEE.load(deps.storage)?,
        })
    }

    /// Returns DAO address
    pub fn query_dao_addr(deps: Deps) -> StdResult<Addr> {
        deps.api.addr_humanize(&DAO.load(deps.storage)?)
    }

    /// Return total number of wallets created
    pub fn query_total(deps: Deps) -> StdResult<u64> {
        TOTAL_CREATED.load(deps.storage)
    }
}

/// Ensures provided addr is the state stored DAO
pub fn ensure_is_dao(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let dao = DAO.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller == dao {
        Ok(())
    } else {
        Err(ContractError::Unauthorized {})
    }
}

/// Ensures provided fixed multisig threshold is valid.
pub fn ensure_is_valid_threshold(guardians: &Guardians) -> Result<(), ContractError> {
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

/// Ensure controller has sent in enough to cover the fee and the initial proxy balance
pub fn ensure_enough_native_funds(
    fee: &Coin,
    proxy_initial_fund: &[Coin],
    multisig_initial_fund: &[Coin],
    sent_fund: &[Coin],
) -> Result<(), ContractError> {
    let init_native_fund = proxy_initial_fund.iter().fold(Uint128::zero(), |acc, c| {
        if c.denom == fee.denom {
            acc + c.amount
        } else {
            acc
        }
    });

    let init_multisig_native_fund = multisig_initial_fund
        .iter()
        .fold(Uint128::zero(), |acc, c| {
            if c.denom == fee.denom {
                acc + c.amount
            } else {
                acc
            }
        });

    let total_native_fund_required = fee.amount + init_native_fund + init_multisig_native_fund;

    let total_sent = sent_fund.iter().fold(Uint128::zero(), |acc, c| {
        if c.denom == fee.denom {
            acc + c.amount
        } else {
            acc
        }
    });

    if total_native_fund_required == total_sent {
        Ok(())
    } else {
        Err(ContractError::InvalidNativeFund(
            total_native_fund_required,
            total_sent,
        ))
    }
}

// Perform security checks to ensure migration message is valid
pub fn ensure_is_valid_migration_msg(
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
                // Ensure Signer of relayed message is the wallet controller
                if wallet_info.controller_addr
                    != pub_key_to_address(
                        &deps.as_ref(),
                        &ADDR_PREFIX.load(deps.storage)?,
                        &tx.controller_pubkey.0,
                    )?
                {
                    return Err(ContractError::InvalidRelayMigrationTx(
                        RelayTxError::IsNotController {},
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
            // Ensure caller is the wallet controller
            if wallet_info.controller_addr != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            cosmwasm_std::from_slice(&msg)?
        }
    };
    Ok(tx_msg)
}

/// Ensure controller has sent in enought funds to cover the claim fee
pub fn ensure_is_enough_claim_fee(deps: Deps, sent_fund: &[Coin]) -> Result<(), ContractError> {
    let claim_fee = CLAIM_FEE.load(deps.storage)?;

    let fund = sent_fund
        .iter()
        .find(|c| c.denom == claim_fee.denom)
        .ok_or_else(|| {
            ContractError::Std(StdError::GenericErr {
                msg: format!("Expected denom fee: {}", claim_fee.denom),
            })
        })?;

    if fund.amount < claim_fee.amount {
        return Err(ContractError::InvalidNativeFund(
            claim_fee.amount,
            fund.amount,
        ));
    }

    Ok(())
}

pub fn handle_proxy_instantion_reply(
    deps: DepsMut,
    env: Env,
    reply: Reply,
) -> Result<Response, ContractError> {
    if let Ok(res) = parse_reply_instantiate_data(reply) {
        let wallet_addr: CanonicalAddr = deps.api.addr_canonicalize(&res.contract_address)?;
        let expiration = Expiration::AtTime(env.block.time)
            .add(DAY.mul(GOVEC_CLAIM_DURATION_DAY_MUL))
            .expect("error defining activate_at");

        GOVEC_CLAIM_LIST.save(deps.storage, wallet_addr.to_vec(), &expiration)?;

        let res = Response::new()
            .add_attribute("action", "Govec claim list updated")
            .add_attribute("proxy_address", res.contract_address);
        Ok(res)
    } else {
        Err(ContractError::ProxyInstantiationError {})
    }
}
