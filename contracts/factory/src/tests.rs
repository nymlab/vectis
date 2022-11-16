#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};
use vectis_wallet::UpdateFeeReq;

#[cfg(feature = "dao-chain")]
use crate::contract::query_govec_addr;
use crate::{
    contract::{
        execute, instantiate, query_code_id, query_dao_addr, query_fees, query_unclaim_wallet_list,
        CodeIdType,
    },
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, UnclaimedWalletList},
};
#[cfg(feature = "remote")]
use {
    crate::contract::query_pending_unclaim_wallet_list,
    crate::msg::QueryMsg,
    crate::state::GOVEC_CLAIM_LIST,
    cosmwasm_std::Api,
    cw_utils::{Expiration, DAY},
    std::ops::{Add, Mul},
    vectis_wallet::GOVEC_CLAIM_DURATION_DAY_MUL,
};

// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    proxy_code_id: u64,
    proxy_multisig_code_id: u64,
    addr_prefix: &str,
    wallet_fee: Coin,
    claim_fee: Coin,
    govec_minter: Option<&str>,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        addr_prefix: addr_prefix.to_string(),
        wallet_fee,
        claim_fee,
        govec_minter: govec_minter.map(|s| s.to_string()),
    };
    let info = mock_info("admin", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();

    let dao = query_dao_addr(deps.as_ref()).unwrap();
    assert_eq!(dao.as_str(), "admin");
}

#[test]
fn initialise_with_no_wallets() {
    let mut deps = mock_dependencies();

    do_instantiate(
        deps.as_mut(),
        0,
        1,
        "wasm",
        coin(1, "ucosm"),
        coin(1, "ucosm"),
        None,
    );

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
    let mut deps = mock_dependencies();
    let info = mock_info("admin", &[]);
    let env = mock_env();
    let initial_code_id = 1111;
    let wallet_fee = coin(1, "ucosm");
    let claim_fee = coin(2, "ucosm");

    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id,
        "wasm",
        wallet_fee.clone(),
        claim_fee.clone(),
        None,
    );

    let fees = query_fees(deps.as_ref()).unwrap();
    assert_eq!(fees.wallet_fee, wallet_fee);
    assert_eq!(fees.claim_fee, claim_fee);

    let new_wallet_fee = coin(3, "ucosm");
    let msg = ExecuteMsg::UpdateConfigFee {
        new_fee: UpdateFeeReq::Wallet(new_wallet_fee.clone()),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "Wallet Fee"), ("New Fee", "3ucosm")]
    );

    let fees = query_fees(deps.as_ref()).unwrap();
    assert_eq!(fees.wallet_fee, new_wallet_fee);

    let new_claim_fee = coin(25, "ucosm");
    let msg = ExecuteMsg::UpdateConfigFee {
        new_fee: UpdateFeeReq::Claim(new_claim_fee.clone()),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "Claim Fee"), ("New Fee", "25ucosm")]
    );

    let fees = query_fees(deps.as_ref()).unwrap();
    assert_eq!(fees.claim_fee, new_claim_fee);
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
        coin(1, "ucosm"),
        None,
    );

    let info = mock_info("admin", &[]);
    let env = mock_env();

    #[cfg(feature = "dao-chain")]
    {
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
    }

    #[cfg(feature = "remote")]
    {
        let msg = ExecuteMsg::UpdateGovecAddr {
            addr: "new_govec".to_string(),
        };

        let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(err, ContractError::NotSupportedByChain {});
    }

    // update admin
    let msg = ExecuteMsg::UpdateDao {
        addr: "new_dao".to_string(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("config", "DAO"), ("New DAO", "new_dao")]
    );

    let new_admin = query_dao_addr(deps.as_ref()).unwrap();
    assert_eq!(new_admin, "new_dao");

    // old admin cannot update addresses
    let msg = ExecuteMsg::UpdateDao {
        addr: "new_dao".to_string(),
    };
    #[cfg(feature = "remote")]
    {
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }

    #[cfg(feature = "dao-chain")]
    {
        let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        let msg = ExecuteMsg::UpdateGovecAddr {
            addr: "new_govec".to_string(),
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});
    }
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
        coin(1, "ucosm"),
        None,
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
        coin(1, "ucosm"),
        None,
    );

    let info = mock_info("non_admin", &[]);
    let env = mock_env();

    let err = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::UpdateConfigFee {
            new_fee: UpdateFeeReq::Wallet(coin(1, "ucosm")),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}
