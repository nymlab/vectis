#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};

use crate::contract::{
    execute, instantiate, query_fee, query_proxy_code_id, query_wallet_list, CodeIdType,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, WalletListResponse};

// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    proxy_code_id: u64,
    proxy_multisig_code_id: u64,
    govec_code_id: u64,
    staking_code_id: u64,
    addr_prefix: &str,
    wallet_fee: Coin,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        govec_code_id,
        staking_code_id,
        addr_prefix: addr_prefix.to_string(),
        wallet_fee,
    };
    let info = mock_info("admin", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
}

#[test]
fn initialise_with_no_wallets() {
    let mut deps = mock_dependencies();

    do_instantiate(deps.as_mut(), 0, 1, 2, 3, "wasm", coin(1, "ucosm"));

    // no wallets to start
    let wallets: WalletListResponse = query_wallet_list(deps.as_ref()).unwrap();
    assert_eq!(wallets.wallets.len(), 0);
}

#[test]
fn initialise_with_correct_code_id() {
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    let initial_multisig_code_id = 2222;
    let initial_govec_code_id = 3333;
    let initial_staking_code_id = 4444;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_multisig_code_id,
        initial_govec_code_id,
        initial_staking_code_id,
        "wasm",
        coin(1, "ucosm"),
    );
    let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(proxy_code_id, initial_code_id);
}

#[test]
fn admin_upgrade_proxy_code_id_works() {
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    let new_code_id = 7777;
    let ty = CodeIdType::Proxy;
    let initial_multisig_code_id = 2222;
    let initial_govec_code_id = 3333;
    let initial_staking_code_id = 4444;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_multisig_code_id,
        initial_govec_code_id,
        initial_staking_code_id,
        "wasm",
        coin(1, "ucosm"),
    );
    let proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(proxy_code_id, initial_code_id);

    let info = mock_info("admin", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::UpdateCodeId {
        ty: ty.clone(),
        new_code_id,
    };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [
            ("config", "Code Id"),
            ("type", &format!("{:?}", ty)),
            ("new Id", &new_code_id.to_string())
        ]
    );

    let new_proxy_code_id = query_proxy_code_id(deps.as_ref()).unwrap();
    assert_eq!(new_proxy_code_id, new_code_id);
}

#[test]
fn admin_update_fee_works() {
    let fee = coin(1, "ucosm");
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    let initial_multisig_code_id = 2222;
    let initial_govec_code_id = 3333;
    let initial_staking_code_id = 4444;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_multisig_code_id,
        initial_govec_code_id,
        initial_staking_code_id,
        "wasm",
        fee.clone(),
    );
    let old_fee = query_fee(deps.as_ref()).unwrap();
    assert_eq!(old_fee, fee);

    let info = mock_info("admin", &[]);
    let env = mock_env();
    let new_update_fee = coin(3, "ucosm");
    let msg = ExecuteMsg::UpdateWalletFee {
        new_fee: new_update_fee.clone(),
    };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "Wallet Fee"), ("New Fee", "3ucosm")]
    );

    let new_fee = query_fee(deps.as_ref()).unwrap();
    assert_eq!(new_fee, new_update_fee);
}
