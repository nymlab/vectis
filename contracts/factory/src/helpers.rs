#[cfg(feature = "dao-chain")]
use crate::contract::GOVEC_REPLY_ID;
use crate::error::ContractError;
use crate::state::{ADDR_PREFIX, ADMIN, GOVEC_CLAIM_LIST, GOVEC_MINTER};
use cosmwasm_std::{
    to_binary, Addr, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw1::CanExecuteResponse;
#[cfg(feature = "dao-chain")]
use vectis_govec::msg::ExecuteMsg::Mint;
pub use vectis_proxy::msg::QueryMsg as ProxyQueryMsg;
#[cfg(feature = "remote")]
use vectis_remote_tunnel::msg::ExecuteMsg::MintGovec;
pub use vectis_wallet::{
    pub_key_to_address, query_verify_cosmos, Guardians, ProxyMigrationTxMsg, RelayTransaction,
    RelayTxError, WalletInfo,
};

/// Ensures provided addr is the state stored ADMIN
pub fn ensure_is_admin(deps: Deps, sender: &str) -> Result<(), ContractError> {
    let admin = ADMIN.load(deps.storage)?;
    let caller = deps.api.addr_canonicalize(sender)?;
    if caller == admin {
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
/// Ensure user has sent in enough to cover the fee and the initial proxy balance
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
                // Ensure Signer of relayed message is the wallet user
                if wallet_info.user_addr
                    != pub_key_to_address(
                        &deps.as_ref(),
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

pub fn ensure_has_govec(deps: Deps) -> Result<CanonicalAddr, ContractError> {
    GOVEC_MINTER
        .may_load(deps.storage)?
        .ok_or(ContractError::GovecNotSet {})
}

#[cfg(feature = "dao-chain")]
pub fn create_mint_msg(govec_minter: String, wallet: String) -> StdResult<SubMsg> {
    Ok(SubMsg::reply_always(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: govec_minter,
            msg: to_binary(&Mint { new_wallet: wallet })?,
            funds: vec![],
        }),
        GOVEC_REPLY_ID,
    ))
}

#[cfg(feature = "remote")]
pub fn create_mint_msg(govec_minter: String, wallet: String) -> StdResult<SubMsg> {
    Ok(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: govec_minter,
        msg: to_binary(&MintGovec {
            wallet_addr: wallet,
        })?,
        funds: vec![],
    })))
}

pub fn handle_govec_minted(
    deps: DepsMut,
    claiming_user: CanonicalAddr,
) -> Result<Response, ContractError> {
    GOVEC_CLAIM_LIST.remove(deps.storage, claiming_user.to_vec());
    let res = Response::new()
        .add_attribute("action", "Govec Minted on DAO Chain")
        .add_attribute("proxy_address", deps.api.addr_humanize(&claiming_user)?);
    Ok(res)
}
