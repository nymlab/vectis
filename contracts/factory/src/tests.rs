#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};

use crate::{
    contract::{
        execute, instantiate, query_admin_addr, query_code_id, query_fee, query_govec_addr,
        query_unclaim_wallet_list, query_wallet_claim_expiration, CodeIdType,
    },
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, UnclaimedWalletList},
};
// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    proxy_code_id: u64,
    proxy_multisig_code_id: u64,
    addr_prefix: &str,
    wallet_fee: Coin,
    govec: Option<String>,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        addr_prefix: addr_prefix.to_string(),
        wallet_fee,
        govec,
    };
    let info = mock_info("admin", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
}

#[test]
fn initialise_with_no_wallets() {
    let mut deps = mock_dependencies();

    do_instantiate(deps.as_mut(), 0, 1, "wasm", coin(1, "ucosm"), None);

    // no wallets to start
    let wallets: UnclaimedWalletList =
        query_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert_eq!(wallets.wallets.len(), 0);
}

#[test]
fn admin_upgrade_code_id_works() {
    let mut deps = mock_dependencies();
    let new_code_id = 7777;
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id + 1,
        "wasm",
        coin(1, "ucosm"),
        None,
    );

    let info = mock_info("admin", &[]);
    let env = mock_env();

    // manual iter
    let tys = vec![CodeIdType::Proxy, CodeIdType::Multisig];

    for (i, t) in tys.iter().enumerate() {
        assert_eq!(
            query_code_id(deps.as_ref(), t.clone()).unwrap(),
            initial_code_id + i as u64
        );
        let response = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateCodeId {
                ty: t.clone(),
                new_code_id: i as u64 + new_code_id,
            },
        )
        .unwrap();
        assert_eq!(
            response.attributes,
            [
                ("config", "Code Id"),
                ("type", &format!("{:?}", t)),
                ("new Id", &(new_code_id + i as u64).to_string())
            ]
        );
    }
}

#[test]
fn admin_update_fee_works() {
    let fee = coin(1, "ucosm");
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id,
        "wasm",
        fee.clone(),
        None,
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

#[test]
fn admin_updates_addresses_work() {
    let fee = coin(1, "ucosm");
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id,
        "wasm",
        fee.clone(),
        None,
    );
    let old_fee = query_fee(deps.as_ref()).unwrap();
    assert_eq!(old_fee, fee);

    let info = mock_info("admin", &[]);
    let env = mock_env();

    // update govec
    let msg = ExecuteMsg::UpdateGovecAddr {
        addr: "new_govec".to_string(),
    };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "Govec Addr"), ("New Addr", "new_govec")]
    );

    let new_govec = query_govec_addr(deps.as_ref()).unwrap();
    assert_eq!(new_govec, "new_govec");

    // update admin
    let msg = ExecuteMsg::UpdateDao {
        addr: "new_admin".to_string(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "Admin"), ("New Admin", "new_admin")]
    );

    let new_admin = query_admin_addr(deps.as_ref()).unwrap();
    assert_eq!(new_admin, "new_admin");

    // old admin cannot update admin or govec anymore
    let msg = ExecuteMsg::UpdateDao {
        addr: "new_admin".to_string(),
    };

    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let msg = ExecuteMsg::UpdateGovecAddr {
        addr: "new_govec".to_string(),
    };

    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn non_admin_update_code_id_fails() {
    let mut deps = mock_dependencies();
    let new_code_id = 7777;
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id + 1,
        "wasm",
        coin(1, "ucosm"),
        None,
    );

    let info = mock_info("non_admin", &[]);
    let env = mock_env();

    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateCodeId {
            ty: CodeIdType::Proxy,
            new_code_id,
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn non_admin_update_fees_fails() {
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id + 1,
        "wasm",
        coin(1, "ucosm"),
        None,
    );

    let info = mock_info("non_admin", &[]);
    let env = mock_env();

    let err = execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        ExecuteMsg::UpdateWalletFee {
            new_fee: coin(3, "ucosm"),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}
