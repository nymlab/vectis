use crate::{
    contract::{
        execute, instantiate, query_code_id, query_dao_addr, query_fees, query_unclaim_wallet_list,
        CodeIdType,
    },
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, UnclaimedWalletList},
};
#[cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, Coin, DepsMut};
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
fn not_found_query() {
    use crate::contract::query;
    let deps = mock_dependencies();
    query(deps.as_ref(), mock_env(), QueryMsg::GovecAddr {}).unwrap_err();
}
#[test]
fn non_admin_cannot_call_minted() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    do_instantiate(
        deps.as_mut(),
        0,
        1,
        "wasm",
        coin(1, "ucosm"),
        coin(1, "ucosm"),
        None,
    );

    //IBC returns  success
    let err = execute(
        deps.as_mut(),
        env,
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
