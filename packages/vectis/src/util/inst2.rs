use crate::types::error::Inst2CalcError;
use cosmwasm_std::{instantiate2_address, Addr, Binary, CodeInfoResponse, Deps, Env};

pub fn calc_instantiate2_addr_from_contract(
    deps: Deps,
    env: &Env,
    code_id: u64,
    salt: &Binary,
) -> Result<Addr, Inst2CalcError> {
    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

    Ok(deps
        .api
        .addr_humanize(&instantiate2_address(&checksum, &creator, salt)?)?)
}
