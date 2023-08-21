use crate::types::error::Inst2CalcError;
use cosmwasm_std::{instantiate2_address, to_binary, Addr, Binary, CodeInfoResponse, Deps, Env};

#[cfg(feature = "multitest")]
use cosmwasm_std::HexBinary;

#[cfg(not(feature = "multitest"))]
pub fn calc_instantiate2_addr_from_contract(
    deps: Deps,
    env: &Env,
    code_id: u64,
    salt: Option<&Binary>,
) -> Result<Addr, Inst2CalcError> {
    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(code_id)?;

    Ok(deps.api.addr_humanize(&instantiate2_address(
        &checksum,
        &creator,
        &salt.unwrap_or(&to_binary(&env.block.time.seconds())?),
    )?)?)
}

#[cfg(feature = "multitest")]
pub fn calc_instantiate2_addr_from_contract(
    deps: Deps,
    env: &Env,
    _code_id: u64,
    salt: Option<&Binary>,
) -> Result<Addr, Inst2CalcError> {
    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    println!("create: {creator:?}");

    let checksum =
        HexBinary::from_hex("58b6db2d844ad5a53252dcb2c80c9a0ea3f8de4aeb184db475909b64fd92f0aa")
            .unwrap();

    let addr = instantiate2_address(
        &checksum,
        &creator,
        &salt.unwrap_or(&to_binary(&env.block.time.seconds())?),
    )?;
    println!("addr: {addr:?}");

    let human_addr = deps.api.addr_humanize(&addr)?;
    println!("human_addr : {human_addr:?}");
    Ok(human_addr)
}
