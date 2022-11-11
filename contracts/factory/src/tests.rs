#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};

#[cfg(feature = "dao-chain")]
use crate::contract::query_govec_addr;
use crate::{
    contract::{
        execute, instantiate, query_code_id, query_dao_addr, query_fee, query_unclaim_wallet_list,
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
    govec_minter: Option<&str>,
) {
    // we do not do integrated tests here so code ids are arbitrary
    let instantiate_msg = InstantiateMsg {
        proxy_code_id,
        proxy_multisig_code_id,
        addr_prefix: addr_prefix.to_string(),
        wallet_fee,
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

    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    #[cfg(feature = "dao-chain")]
    {
        let msg = ExecuteMsg::UpdateGovecAddr {
            addr: "new_govec".to_string(),
        };

        let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
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

#[test]
#[cfg(feature = "remote")]
fn not_found_query() {
    use crate::contract::query;
    let deps = mock_dependencies();
    query(deps.as_ref(), mock_env(), QueryMsg::GovecAddr {}).unwrap_err();
}
#[test]
#[cfg(feature = "remote")]
fn non_admin_cannot_call_minted() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    do_instantiate(deps.as_mut(), 0, 1, "wasm", coin(1, "ucosm"), None);

    //IBC returns  success
    let err = execute(
        deps.as_mut(),
        env.clone(),
        mock_info("NOT_ADMIN", &[]),
        ExecuteMsg::GovecMinted {
            success: true,
            wallet_addr: "wallet".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {})
}

#[test]
#[cfg(feature = "remote")]
fn handles_minted_submsg() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let initial_code_id = 1111;
    do_instantiate(
        deps.as_mut(),
        initial_code_id,
        initial_code_id + 1,
        "wasm",
        coin(1, "ucosm"),
        None,
    );

    // We pretent that a wallet has been created
    let success_addr = deps.api.addr_canonicalize("yes_wallet").unwrap();
    let failed_addr = deps.api.addr_canonicalize("no_wallet").unwrap();

    GOVEC_CLAIM_LIST
        .save(
            &mut deps.storage,
            success_addr.to_vec(),
            &Expiration::AtHeight(env.block.height + 100),
        )
        .unwrap();

    GOVEC_CLAIM_LIST
        .save(
            &mut deps.storage,
            failed_addr.to_vec(),
            &Expiration::AtHeight(env.block.height + 100),
        )
        .unwrap();

    let claim_list = query_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert_eq!(claim_list.wallets.len(), 2);

    // wallet calls to be claimed
    let info = mock_info("yes_wallet", &[]);
    execute(deps.as_mut(), env.clone(), info, ExecuteMsg::ClaimGovec {}).unwrap();

    let claim_list = query_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert_eq!(claim_list.wallets.len(), 1);

    let pending_claim_list = query_pending_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert_eq!(pending_claim_list[0].as_str(), "yes_wallet");

    //IBC returns  success
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("admin", &[]),
        ExecuteMsg::GovecMinted {
            success: true,
            wallet_addr: "yes_wallet".to_string(),
        },
    )
    .unwrap();

    let pending_claim_list = query_pending_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert!(pending_claim_list.is_empty());

    //IBC returns failed
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("admin", &[]),
        ExecuteMsg::GovecMinted {
            success: false,
            wallet_addr: "no_wallet".to_string(),
        },
    )
    .unwrap();

    let pending_claim_list = query_pending_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert!(pending_claim_list.is_empty());

    let claim_list = query_unclaim_wallet_list(deps.as_ref(), None, None).unwrap();
    assert_eq!(claim_list.wallets.len(), 1);

    // Expiration is updated
    let expiration = Expiration::AtTime(env.block.time)
        .add(DAY.mul(GOVEC_CLAIM_DURATION_DAY_MUL))
        .unwrap();
    assert_eq!(claim_list.wallets[0].1, expiration);
}
