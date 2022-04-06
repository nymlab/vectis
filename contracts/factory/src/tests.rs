#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{DepsMut, Uint128};

use crate::contract::{execute, instantiate, query_proxy_code_id, query_wallet_list};
use crate::msg::{ExecuteMsg, InstantiateMsg, WalletListResponse};

// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    proxy_code_id: u64,
    proxy_multisig_code_id: u64,
    addr_prefix: &str,
    coin_denom: &str,
    wallet_fee: Uint128,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        addr_prefix: addr_prefix.to_string(),
        coin_denom: coin_denom.to_string(),
        wallet_fee: wallet_fee,
    };
    let info = mock_info("admin", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
}

#[test]
fn initialise_with_no_wallets() {
    let mut deps = mock_dependencies();

    do_instantiate(deps.as_mut(), 0, 1, "wasm", "ucosm", Uint128::new(1));

    // no wallets to start
    let wallets: WalletListResponse = query_wallet_list(deps.as_ref()).unwrap();
    assert_eq!(wallets.wallets.len(), 0);
}

#[test]
fn initialise_with_correct_code_id() {
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    let initial_multisig_code_id = 2222;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_multisig_code_id,
        "wasm",
        "ucosm",
        Uint128::new(1),
    );
    let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(proxy_code_id, initial_code_id);
}

#[test]
fn admin_upgrade_proxy_code_id_works() {
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    let new_code_id = 2222;
    let initial_multisig_code_id = 2222;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_multisig_code_id,
        "wasm",
        "ucosm",
        Uint128::new(1),
    );
    let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(proxy_code_id, initial_code_id);

    let info = mock_info("admin", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::UpdateProxyCodeId { new_code_id };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [
            ("config", "Proxy Code Id"),
            ("proxy_code_id", &new_code_id.to_string())
        ]
    );

    let new_proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(new_proxy_code_id, new_code_id);
}
