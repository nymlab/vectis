use std::ops::Add;

pub use crate::contract::*;
pub use crate::enumerable::*;
pub use crate::error::*;
pub use crate::msg::*;
pub use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
};
pub use cosmwasm_std::{
    coins, from_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, StdError, SubMsg, Uint128, WasmMsg,
};
pub use cw20::{
    BalanceResponse, Cw20Coin, Cw20ReceiveMsg, Logo, MarketingInfoResponse, TokenInfoResponse,
};
pub use cw20_stake::contract::{query_download_logo, query_marketing_info};
pub use vectis_wallet::{MintResponse, UpdateAddrReq};

pub const FACTORY: &str = "factory";
pub const DAO_ADDR: &str = "dao";

pub fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
pub fn do_instantiate(
    mut deps: DepsMut,
    addr: Vec<&str>,
    amount: Vec<Uint128>,
    factory: Option<&str>,
    mint_cap: Option<Uint128>,
    mint_amount: Uint128,
    marketing: Option<MarketingInfoResponse>,
) -> TokenInfoResponse {
    let mut coins: Vec<Cw20Coin> = Vec::new();
    let mut total_supply = Uint128::zero();
    for (i, &el) in addr.iter().enumerate() {
        coins.push(Cw20Coin {
            address: el.to_string(),
            amount: amount[i],
        });
        total_supply = amount[i] + total_supply;
    }
    let instantiate_msg = InstantiateMsg {
        name: "Auto Gen".to_string(),
        symbol: "AUTO".to_string(),
        initial_balances: coins,
        staking_addr: None,
        mint_cap,
        mint_amount,
        factory: factory.map(|e| e.to_string()),
        marketing,
    };

    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();

    let meta = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(
        meta,
        TokenInfoResponse {
            name: "Auto Gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 0,
            total_supply
        }
    );

    // Test minters set correctly in multitest

    assert_eq!(query_dao(deps.as_ref()).unwrap(), DAO_ADDR);
    meta
}

#[test]
fn can_instantiate_accounts() {
    let mut deps = mock_dependencies();
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(511223344);
    let instantiate_msg = InstantiateMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        initial_balances: vec![
            Cw20Coin {
                address: "addr0000".into(),
                amount,
            },
            Cw20Coin {
                address: "addr0011".into(),
                amount,
            },
        ],
        staking_addr: None,
        factory: None,
        mint_cap: Some(limit),
        mint_amount: Uint128::new(2),
        marketing: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Cash Token".to_string(),
            symbol: "CASH".to_string(),
            decimals: 0,
            total_supply: amount.add(amount),
        }
    );
    assert_eq!(get_balance(deps.as_ref(), "addr0000"), amount);
    assert_eq!(get_balance(deps.as_ref(), "addr0011"), amount);
}

#[test]
fn cannot_mint_over_cap() {
    let mut deps = mock_dependencies();
    let amount = Uint128::new(11223344);
    let limit = Uint128::new(11223300);
    let instantiate_msg = InstantiateMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        initial_balances: vec![Cw20Coin {
            address: String::from("addr0000"),
            amount,
        }],
        factory: None,
        mint_cap: Some(limit),
        mint_amount: Uint128::new(2),
        staking_addr: None,
        marketing: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let err = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Initial supply greater than cap").into()
    );
}

#[test]
fn query_joined_works() {
    let mut deps = mock_dependencies();
    let genesis = String::from("genesis");
    let not_genesis = String::from("not_genesis");
    let amount = Uint128::new(10);
    let limit = Uint128::new(12);

    do_instantiate(
        deps.as_mut(),
        vec![genesis.as_str()],
        vec![amount],
        Some(FACTORY),
        Some(limit),
        Uint128::new(2),
        None,
    );

    assert_eq!(
        query_balance(deps.as_ref(), genesis.clone())
            .unwrap()
            .balance,
        amount
    );
    assert_eq!(
        query_balance(deps.as_ref(), not_genesis.clone())
            .unwrap()
            .balance,
        Uint128::zero()
    );

    assert!(query_balance_joined(deps.as_ref(), genesis)
        .unwrap()
        .is_some());
    assert!(query_balance_joined(deps.as_ref(), not_genesis)
        .unwrap()
        .is_none());
}

#[test]
fn dao_can_update_staking_addr() {
    let mut deps = mock_dependencies();
    let genesis = String::from("genesis");
    let amount = Uint128::new(0);
    let limit = Uint128::new(12);

    let new_staking = String::from("new_staking");

    do_instantiate(
        deps.as_mut(),
        vec![genesis.as_str()],
        vec![amount],
        Some(FACTORY),
        Some(limit),
        Uint128::new(2),
        None,
    );

    let msg = ExecuteMsg::UpdateConfigAddr {
        new_addr: UpdateAddrReq::Staking(new_staking.clone()),
    };

    // only dao can update staking
    let info = mock_info(FACTORY, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // dao can update staking
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(query_staking(deps.as_ref()).unwrap(), new_staking);
}

#[test]
fn dao_can_update_mint_amount() {
    let mut deps = mock_dependencies();
    let new_amount = Uint128::new(123);

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        Some(FACTORY),
        None,
        Uint128::new(2),
        None,
    );

    let msg = ExecuteMsg::UpdateMintAmount { new_amount };

    // only dao can update mint data
    let info = mock_info(FACTORY, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // dao can update mint data
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(query_mint_amount(deps.as_ref()).unwrap(), new_amount);
}

#[test]
fn queries_work() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = "addr0001";
    let amount1 = Uint128::new(12340000);
    let limit = Uint128::new(999999900);

    let expected = do_instantiate(
        deps.as_mut(),
        vec![addr1],
        vec![amount1],
        Some(FACTORY),
        Some(limit),
        Uint128::new(2),
        None,
    );

    // check meta query
    let loaded = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(expected, loaded);

    let env = mock_env();
    // check balance query (full)
    let data = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Balance {
            address: addr1.to_string(),
        },
    )
    .unwrap();
    let loaded: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(loaded.balance, amount1);

    // check balance query (empty)
    let data = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Balance {
            address: String::from("addr0002-not-a-wallet"),
        },
    )
    .unwrap();
    let loaded: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(loaded.balance, Uint128::new(0));

    // check balance query (empty)
    let data = query(
        deps.as_ref(),
        env,
        QueryMsg::Joined {
            address: String::from("addr0002-not-a-wallet"),
        },
    )
    .unwrap();
    let loaded: Option<BalanceResponse> = from_binary(&data).unwrap();
    assert_eq!(loaded, None);
}

#[test]
fn burn() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let all = Uint128::new(10);
    do_instantiate(
        deps.as_mut(),
        vec![DAO_ADDR],
        vec![all],
        Some(FACTORY),
        None,
        Uint128::new(2),
        None,
    );

    let initial_total_supply = query_token_info(deps.as_ref()).unwrap().total_supply;

    // valid burn reduces total supply and remove account from BALANCES
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(1),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let remainder = initial_total_supply.checked_sub(Uint128::new(1)).unwrap();
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        remainder
    );
    let data = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Balance {
            address: DAO_ADDR.to_string(),
        },
    )
    .unwrap();
    let balance: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(balance.balance, Uint128::new(9));

    // cannot burn too much
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(11),
    };
    execute(deps.as_mut(), env, info, msg).unwrap_err();

    // cannot burn if not DAO
    let info = mock_info("NOT_DAO", &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {
        amount: Uint128::new(1),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {})
}

#[test]
fn exit() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let all = Uint128::new(10);
    let addr1 = "addr1";
    do_instantiate(
        deps.as_mut(),
        vec![addr1],
        vec![all],
        Some(FACTORY),
        None,
        Uint128::new(2),
        None,
    );
    let init_dao_balance = query_balance(deps.as_ref(), DAO_ADDR.to_string()).unwrap();

    // cannot transfer to burnt wallet
    let info = mock_info(addr1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Exit { relayed_from: None };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(res.attributes, [("action", "exit"), ("addr", addr1)]);
    let post_dao_balance = query_balance(deps.as_ref(), DAO_ADDR.to_string()).unwrap();

    assert_eq!(all, post_dao_balance.balance - init_dao_balance.balance);

    assert_eq!(
        None,
        query_balance_joined(deps.as_ref(), addr1.to_string()).unwrap()
    )
}

#[test]
fn query_all_accounts_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // insert order and lexicographical order are different
    let acct1 = String::from("acct01");
    let acct2 = String::from("zebra");
    let acct3 = String::from("nice");
    let acct4 = String::from("aaaardvark");
    let expected_order = [
        acct4.clone(),
        acct1.clone(),
        "dao".to_string(),
        acct3.clone(),
        acct2.clone(),
    ];

    do_instantiate(
        deps.as_mut(),
        vec![
            acct1.as_str(),
            acct2.as_str(),
            acct3.as_str(),
            acct4.as_str(),
        ],
        vec![
            Uint128::new(1),
            Uint128::new(1),
            Uint128::new(1),
            Uint128::new(1),
        ],
        Some(FACTORY),
        None,
        Uint128::new(2),
        None,
    );

    // make sure we get the proper results
    let accounts = query_all_accounts(deps.as_ref(), None, None).unwrap();
    assert_eq!(accounts.accounts, expected_order);

    // let's do pagination
    let accounts = query_all_accounts(deps.as_ref(), None, Some(2)).unwrap();
    assert_eq!(accounts.accounts, expected_order[0..2].to_vec());

    let accounts =
        query_all_accounts(deps.as_ref(), Some(accounts.accounts[1].clone()), Some(1)).unwrap();
    assert_eq!(accounts.accounts, expected_order[2..3].to_vec());

    let accounts =
        query_all_accounts(deps.as_ref(), Some(accounts.accounts[0].clone()), Some(777)).unwrap();
    assert_eq!(accounts.accounts, expected_order[3..].to_vec());
}

#[test]
fn query_marketing_info_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let project = "website".to_string();
    let description = "description".to_string();

    let marketing_msg = MarketingInfoResponse {
        project: Some(project.clone()),
        description: Some(description.clone()),
        marketing: Some(Addr::unchecked(DAO_ADDR)),
        logo: None,
    };

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        Some(FACTORY),
        None,
        Uint128::new(2),
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.project.unwrap(), project);
    assert_eq!(marketing_info.description.unwrap(), description);
}

#[test]
fn query_download_logo_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        None,
        None,
        Uint128::new(2),
        None,
    );

    let logo = query_download_logo(deps.as_ref()).expect_err("NotFound");

    assert_eq!(logo.to_string(), "cw20::logo::Logo not found");
}

#[test]
fn execute_update_marketing_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();

    let project = "website".to_string();
    let description = "description".to_string();

    let marketing_msg = MarketingInfoResponse {
        project: Some(project.clone()),
        description: Some(description.clone()),
        marketing: Some(Addr::unchecked(DAO_ADDR)),
        logo: None,
    };

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        None,
        None,
        Uint128::new(2),
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.project.unwrap(), project);
    assert_eq!(marketing_info.description.unwrap(), description);

    let project = "new-website".to_string();
    let description = "new-description".to_string();

    let res = govec_execute_update_marketing(
        deps.as_mut(),
        env,
        info,
        Some(project.clone()),
        Some(description.clone()),
        None,
    )
    .unwrap();

    assert_eq!(res.attributes, [("action", "update_marketing")]);

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();

    assert_eq!(marketing_info.project.unwrap(), project);
    assert_eq!(marketing_info.description.unwrap(), description);
}

#[test]
fn execute_update_logo_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();

    let project = "website".to_string();
    let description = "description".to_string();

    let marketing_msg = MarketingInfoResponse {
        project: Some(project),
        description: Some(description),
        marketing: Some(Addr::unchecked(DAO_ADDR)),
        logo: None,
    };

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        None,
        None,
        Uint128::new(2),
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.logo.is_none(), true);

    let logo = Logo::Url("website".to_string());

    let res = govec_execute_upload_logo(deps.as_mut(), env, info, logo).unwrap();

    assert_eq!(res.attributes, [("action", "upload_logo")]);

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.logo.is_some(), true);
}
