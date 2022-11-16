use cosmwasm_std::{
    to_binary, BankMsg, CosmosMsg, Deps, DepsMut, Env, Response, StdResult, SubMsg, WasmMsg,
};
use std::ops::{Add, Mul};

use cw_utils::{Expiration, DAY};

use vectis_wallet::{RemoteTunnelExecuteMsg, RemoteTunnelPacketMsg, GOVEC_CLAIM_DURATION_DAY_MUL};

use crate::{
    state::{CLAIM_FEE, DAO, GOVEC_CLAIM_LIST, PENDING_CLAIM_LIST},
    ContractError,
};

pub fn create_mint_msg(deps: Deps, wallet: String) -> StdResult<SubMsg> {
    Ok(SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps
            .api
            .addr_humanize(&DAO.load(deps.storage)?)?
            .to_string(),
        msg: to_binary(&RemoteTunnelExecuteMsg::DaoActions {
            msg: RemoteTunnelPacketMsg::MintGovec {
                wallet_addr: wallet,
            },
        })?,
        funds: vec![],
    })))
}

pub fn handle_govec_minted(deps: DepsMut, wallet: String) -> Result<Response, ContractError> {
    let claiming_user = deps.api.addr_canonicalize(&wallet)?;
    PENDING_CLAIM_LIST.remove(deps.storage, claiming_user.to_vec());

    let fee = CLAIM_FEE.load(deps.storage)?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: deps.api.addr_humanize(&DAO.load(deps.storage)?)?.into(),
        amount: vec![fee],
    });

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "Govec Minted on DAO Chain")
        .add_attribute("proxy_address", wallet);
    Ok(res)
}

pub fn handle_govec_mint_failed(
    deps: DepsMut,
    env: Env,
    wallet: String,
) -> Result<Response, ContractError> {
    let claiming_user = deps.api.addr_canonicalize(&wallet)?;
    PENDING_CLAIM_LIST.remove(deps.storage, claiming_user.to_vec());
    let expiration = Expiration::AtTime(env.block.time)
        .add(DAY.mul(GOVEC_CLAIM_DURATION_DAY_MUL))
        .expect("error defining activate_at");
    GOVEC_CLAIM_LIST.save(deps.storage, claiming_user.to_vec(), &expiration)?;

    let fee = CLAIM_FEE.load(deps.storage)?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: wallet.clone(),
        amount: vec![fee],
    });

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "Govec Mint failed on DAO Chain")
        .add_attribute("action", "Renewed Claim expiration")
        .add_attribute("proxy_address", wallet);
    Ok(res)
}
