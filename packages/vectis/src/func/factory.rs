use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};

use cw1::CanExecuteResponse;

use crate::factory_state::*;
use crate::{
    pub_key_to_address, query_verify_cosmos, CodeIdType, FactoryError as ContractError, Guardians,
    ProxyMigrationTxMsg, ProxyQueryMsg, RelayTxError,
    WalletFactoryInstantiateMsg as InstantiateMsg, WalletInfo,
};

pub fn factory_instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin_addr = deps.api.addr_canonicalize(info.sender.as_ref())?;

    DEPLOYER.save(deps.storage, &admin_addr)?;
    PROXY_CODE_ID.save(deps.storage, &msg.proxy_code_id)?;
    PROXY_MULTISIG_CODE_ID.save(deps.storage, &msg.proxy_multisig_code_id)?;
    TOTAL_CREATED.save(deps.storage, &0)?;
    ADDR_PREFIX.save(deps.storage, &msg.addr_prefix)?;
    WALLET_FEE.save(deps.storage, &msg.wallet_fee)?;

    let event = Event::new("vectis.factory.v1.MsgInstantiate")
        .add_attribute("contract_address", env.contract.address);

    Ok(Response::new().add_event(event))
}

pub mod factory_execute {
    use super::*;

    use crate::{
        CreateWalletMsg, FeeType, MigrationMsgError, ProxyInstantiateMsg, ProxyMigrateMsg,
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
            let msg = SubMsg::new(instantiate_msg);

            let event = Event::new("vectis.factory.v1.MsgCreateWallet")
                .add_attribute("wallet_id", next_id.to_string());

            let res = Response::new().add_submessage(msg).add_event(event);

            TOTAL_CREATED.save(deps.storage, &next_id)?;

            // Send native tokens to deployer
            if fee.amount != Uint128::zero() {
                let to_address = deps
                    .api
                    .addr_humanize(&DEPLOYER.load(deps.storage)?)?
                    .to_string();

                // Direct transfer to Deployer
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

            let event = Event::new("vectis.factory.v1.MsgMigrateWallet")
                .add_attribute("wallet_address", contract_addr)
                .add_attribute("code_id", new_code_id.to_string());

            Ok(Response::new().add_message(tx_msg).add_event(event))
        } else {
            Err(ContractError::InvalidMigrationMsg(
                MigrationMsgError::InvalidWasmMsg,
            ))
        }
    }

    /// Updates the latest code id for the supported `wallet_proxy`
    pub fn update_code_id(
        deps: DepsMut,
        info: MessageInfo,
        ty: CodeIdType,
        new_code_id: u64,
    ) -> Result<Response, ContractError> {
        ensure_is_deployer(deps.as_ref(), info.sender.as_ref())?;
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

        let event = Event::new("vectis.factory.v1.MsgUpdateCodeId")
            .add_attribute("type", format!("{ty:?}"))
            .add_attribute("code_id", new_code_id.to_string());

        Ok(Response::new().add_event(event))
    }

    pub fn update_config_fee(
        deps: DepsMut,
        info: MessageInfo,
        ty: FeeType,
        new_fee: Coin,
    ) -> Result<Response, ContractError> {
        ensure_is_deployer(deps.as_ref(), info.sender.as_str())?;

        match ty {
            FeeType::Wallet => {
                WALLET_FEE.save(deps.storage, &new_fee)?;
            }
        };

        let event = Event::new("vectis.factory.v1.MsgUpdateConfigFee")
            .add_attribute("type", format!("{ty:?}"))
            .add_attribute("amount", new_fee.amount.to_string())
            .add_attribute("denom", new_fee.denom);

        Ok(Response::new().add_event(event))
    }

    pub fn update_deployer_addr(
        deps: DepsMut,
        info: MessageInfo,
        addr: String,
    ) -> Result<Response, ContractError> {
        ensure_is_deployer(deps.as_ref(), info.sender.as_str())?;
        DEPLOYER.save(deps.storage, &deps.api.addr_canonicalize(&addr)?)?;

        let event = Event::new("vectis.factory.v1.MsgUpdateDaoAddr").add_attribute("address", addr);

        Ok(Response::new().add_event(event))
    }
}

pub mod factory_queries {
    use super::*;

    use crate::FeesResponse;

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
        })
    }

    /// Returns Deployer address
    pub fn query_deployer(deps: Deps) -> StdResult<Addr> {
        deps.api.addr_humanize(&DEPLOYER.load(deps.storage)?)
    }

    /// Return total number of wallets created
    pub fn query_total(deps: Deps) -> StdResult<u64> {
        TOTAL_CREATED.load(deps.storage)
    }
}

/// Ensures provided addr is the state stored DEPLOYER
pub fn ensure_is_deployer(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let deployer = DEPLOYER.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller == deployer {
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
