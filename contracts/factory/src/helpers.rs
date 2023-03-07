use cosmwasm_std::{to_binary, BankMsg, CosmosMsg, Deps, DepsMut, Response, SubMsg, WasmMsg};

use crate::state::{CLAIM_FEE, DAO, GOVEC_CLAIM_LIST, ITEMS};
use crate::ContractError;

use vectis_govec::msg::ExecuteMsg::Mint;
use vectis_wallet::{DaoActors, GOVEC_REPLY_ID};

pub fn create_mint_msg(deps: Deps, wallet: String) -> Result<SubMsg, ContractError> {
    let dao = DAO.load(deps.storage)?;
    let govec = ITEMS
        .query(
            &deps.querier,
            deps.api.addr_humanize(&dao)?,
            DaoActors::Govec.to_string(),
        )?
        .ok_or(ContractError::GovecNotSet {})?;
    Ok(SubMsg::reply_always(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: govec,
            msg: to_binary(&Mint { new_wallet: wallet })?,
            funds: vec![],
        }),
        GOVEC_REPLY_ID,
    ))
}

pub fn handle_govec_minted(deps: DepsMut, wallet: String) -> Result<Response, ContractError> {
    let claiming_controller = deps.api.addr_canonicalize(&wallet)?;
    GOVEC_CLAIM_LIST.remove(deps.storage, claiming_controller.to_vec());

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
