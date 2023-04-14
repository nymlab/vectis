use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};

use vectis_wallet::factory_queries::{query_code_id, query_fees, query_total};
use vectis_wallet::CodeIdType;
use vectis_wallet::{factory_queries::query_deployer, FeeType};

use crate::{
    contract::{execute, instantiate},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
};

// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    proxy_code_id: u64,
    proxy_multisig_code_id: u64,
    addr_prefix: &str,
    wallet_fee: Coin,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        addr_prefix: addr_prefix.to_string(),
        wallet_fee,
    };
    let info = mock_info("admin", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();

    let dao = query_deployer(deps.as_ref()).unwrap();
    assert_eq!(dao.as_str(), "admin");
}

#[test]
fn initialise_with_no_wallets() {
    let mut deps = mock_dependencies();

    do_instantiate(deps.as_mut(), 0, 1, "wasm", coin(1, "ucosm"));

    // no wallets to start
    let wallets = query_total(deps.as_ref()).unwrap();
    assert_eq!(wallets, 0);
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
            response.events[0].attributes,
            [
                ("type", &format!("{t:?}")),
                ("code_id", &(new_code_id + i as u64).to_string())
            ]
        );
    }
}

#[test]
fn admin_update_fee_works() {
    let mut deps = mock_dependencies();
    let info = mock_info("admin", &[]);
    let env = mock_env();
    let initial_code_id = 1111;
    let wallet_fee = coin(1, "ucosm");

    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id,
        "wasm",
        wallet_fee.clone(),
    );

    let fees = query_fees(deps.as_ref()).unwrap();
    assert_eq!(fees.wallet_fee, wallet_fee);

    let new_wallet_fee = coin(3, "ucosm");
    let msg = ExecuteMsg::UpdateConfigFee {
        ty: FeeType::Wallet,
        new_fee: new_wallet_fee.clone(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [
            ("type", &format!("{:?}", FeeType::Wallet)),
            ("amount", &new_wallet_fee.amount.to_string()),
            ("denom", &new_wallet_fee.denom)
        ]
    );

    let fees = query_fees(deps.as_ref()).unwrap();
    assert_eq!(fees.wallet_fee, new_wallet_fee);
}

#[test]
fn admin_updates_addresses_work() {
    let fee = coin(1, "ucosm");
    let mut deps = mock_dependencies();
    let initial_code_id = 1111;
    do_instantiate(deps.as_mut(), initial_code_id, initial_code_id, "wasm", fee);

    let info = mock_info("admin", &[]);
    let env = mock_env();

    // update admin
    let msg = ExecuteMsg::UpdateDeployer {
        addr: "new_dao".to_string(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.events[0].attributes, [("address", "new_dao")]);

    let new_admin = query_deployer(deps.as_ref()).unwrap();
    assert_eq!(new_admin, "new_dao");

    // old admin cannot update addresses
    let msg = ExecuteMsg::UpdateDeployer {
        addr: "new_dao".to_string(),
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
    );

    let info = mock_info("non_admin", &[]);
    let env = mock_env();

    let err = execute(
        deps.as_mut(),
        env,
        info,
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
    );

    let info = mock_info("non_admin", &[]);
    let env = mock_env();

    let err = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateConfigFee {
            ty: FeeType::Wallet,
            new_fee: coin(1, "ucosm"),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}
