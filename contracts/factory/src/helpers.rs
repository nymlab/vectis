use cosmwasm_std::{
    to_binary, BankMsg, CanonicalAddr, CosmosMsg, Deps, DepsMut, Response, StdResult, SubMsg,
    WasmMsg,
};

use crate::state::{CLAIM_FEE, DAO, GOVEC_CLAIM_LIST, GOVEC_MINTER};
use crate::ContractError;

use vectis_govec::msg::ExecuteMsg::Mint;
use vectis_wallet::GOVEC_REPLY_ID;

pub fn ensure_has_govec(deps: Deps) -> Result<CanonicalAddr, ContractError> {
    GOVEC_MINTER
        .may_load(deps.storage)?
        .ok_or(ContractError::GovecNotSet {})
}

pub fn create_mint_msg(deps: Deps, wallet: String) -> StdResult<SubMsg> {
    Ok(SubMsg::reply_always(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps
                .api
                .addr_humanize(&GOVEC_MINTER.load(deps.storage)?)?
                .to_string(),
            msg: to_binary(&Mint { new_wallet: wallet })?,
            funds: vec![],
        }),
        GOVEC_REPLY_ID,
    ))
}

pub fn handle_govec_minted(deps: DepsMut, wallet: String) -> Result<Response, ContractError> {
    let claiming_user = deps.api.addr_canonicalize(&wallet)?;
    GOVEC_CLAIM_LIST.remove(deps.storage, claiming_user.to_vec());

    let fee = CLAIM_FEE.load(deps.storage)?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: deps
            .api
            .addr_humanize(&DAO.load(deps.storage)?)?
            .to_string(),
        amount: vec![fee],
    });

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "Govec Minted on DAO Chain")
        .add_attribute("proxy_address", wallet);
    Ok(res)
}
