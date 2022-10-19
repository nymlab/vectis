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
    BalanceResponse, Cw20Coin, Cw20ReceiveMsg, MarketingInfoResponse, TokenInfoResponse,
};
pub use cw20_stake::contract::{query_download_logo, query_marketing_info};

pub const STAKE_ADDR: &str = "staker";
pub const FACTORY: &str = "factory";
pub const DAO_ADDR: &str = "dao";
pub const DAO_TUNNEL: &str = "dao-tunnel";

pub fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

// this will set up the instantiation for other tests
pub fn do_instantiate(
    mut deps: DepsMut,
    addr: Vec<&str>,
    amount: Vec<Uint128>,
    factory: Option<&str>,
    dao_tunnel: Option<&str>,
    mint_cap: Option<Uint128>,
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
        factory: factory.map(|e| e.to_string()),
        dao_tunnel: dao_tunnel.map(|e| e.to_string()),
        marketing,
    };

    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    instantiate(deps.branch(), env.clone(), info, instantiate_msg).unwrap();

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

    let minters = if factory.is_none() && dao_tunnel.is_none() {
        None
    } else {
        let mut v = Vec::new();
        if let Some(daot) = dao_tunnel {
            v.push(daot.to_string());
        }
        if let Some(f) = factory {
            v.push(f.to_string());
        }
        Some(v)
    };

    let mr = MintResponse {
        minters,
        cap: mint_cap,
    };

    assert_eq!(query_minter(deps.as_ref()).unwrap(), mr);
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
        dao_tunnel: None,
        mint_cap: Some(limit),
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
        dao_tunnel: None,
        mint_cap: Some(limit),
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
        Some(DAO_TUNNEL),
        Some(limit),
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
        Some(DAO_TUNNEL),
        Some(limit),
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
fn dao_can_update_dao_addr_and_transfer_tokens() {
    let mut deps = mock_dependencies();
    let dao_balance = Uint128::new(5);
    let limit = Uint128::new(12);

    let new_dao = String::from("new_dao");

    do_instantiate(
        deps.as_mut(),
        vec![DAO_ADDR],
        vec![dao_balance],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        Some(limit),
        None,
    );
    assert_eq!(
        query_balance(deps.as_ref(), DAO_ADDR.to_string())
            .unwrap()
            .balance,
        Uint128::new(5)
    );
    let msg = ExecuteMsg::UpdateConfigAddr {
        new_addr: UpdateAddrReq::Dao(new_dao.clone()),
    };

    // only dao can update DAO
    let info = mock_info(FACTORY, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(query_dao(deps.as_ref()).unwrap(), new_dao);
    assert_eq!(
        query_balance(deps.as_ref(), DAO_ADDR.to_string())
            .unwrap()
            .balance,
        Uint128::new(0)
    );
    assert_eq!(
        query_balance(deps.as_ref(), new_dao.to_string())
            .unwrap()
            .balance,
        Uint128::new(5)
    );
}

#[test]
fn dao_can_update_mint_cap() {
    let mut deps = mock_dependencies();
    let new_mint_cap = Some(Uint128::new(123));

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    let msg = ExecuteMsg::UpdateMintCap { new_mint_cap };

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
    assert_eq!(query_minter(deps.as_ref()).unwrap().cap, new_mint_cap);
}

#[test]
fn can_mint_by_minter() {
    let mut deps = mock_dependencies();

    let genesis = String::from("genesis");
    let amount = Uint128::new(0);
    let limit = Uint128::new(2);

    do_instantiate(
        deps.as_mut(),
        vec![],
        vec![],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        Some(limit),
        None,
    );

    // minter can mint coins to some winner
    let winner = String::from("lucky");
    let msg = ExecuteMsg::Mint {
        new_wallet: winner.clone(),
    };

    // Others cannot mint
    let info = mock_info("others", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // Factory can mint
    let info = mock_info(FACTORY, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis.clone()), amount);
    assert_eq!(get_balance(deps.as_ref(), winner.clone()), Uint128::new(1));

    // DAO Tunnel can also mint
    let info = mock_info(DAO_TUNNEL, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg.clone()).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), amount);
    assert_eq!(get_balance(deps.as_ref(), winner.clone()), Uint128::new(2));

    // but if it exceeds cap, it fails cap is enforced
    let info = mock_info(FACTORY, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::CannotExceedCap {});
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
        Some(DAO_TUNNEL),
        Some(limit),
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
fn transfer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let not_wallet = String::from("not_wallet");
    let amount1 = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);

    do_instantiate(
        deps.as_mut(),
        vec![addr1.as_str(), addr2.as_str()],
        vec![amount1, Uint128::zero()],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    // cannot transfer nothing
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: Uint128::zero(),
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // cannot send more than we have
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: too_much,
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send from empty account
    let info = mock_info(not_wallet.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: transfer,
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send to non-existing accounts
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: not_wallet.clone(),
        amount: transfer,
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
    assert_eq!(get_balance(deps.as_ref(), addr1.clone()), amount1);
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        amount1
    );

    // valid transfer, aka vote delegation
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: transfer,
        remote_from: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        res.attributes,
        [
            ("action", "transfer"),
            ("from", &addr1),
            ("to", &addr2),
            ("amount", &transfer.to_string())
        ]
    );

    assert_eq!(get_balance(deps.as_ref(), addr2.clone()), transfer);
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        amount1
    );
}

#[test]
fn burn() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let addr3 = String::from("addr0003");
    let right_amount = Uint128::from(1u8);
    let too_little = Uint128::zero();
    let too_much = Uint128::from(2u8);

    do_instantiate(
        deps.as_mut(),
        vec![addr1.as_str(), addr2.as_str(), addr3.as_str()],
        vec![right_amount, too_little, too_much],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    let initial_total_supply = query_token_info(deps.as_ref()).unwrap().total_supply;

    // valid burn reduces total supply and remove account from BALANCES
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn { remote_from: None };
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let remainder = initial_total_supply.checked_sub(right_amount).unwrap();
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        remainder
    );
    let data = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: addr1.clone(),
        },
    )
    .unwrap();
    let balance: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(balance.balance, Uint128::new(0));

    // cannot transfer to burnt wallet
    let info = mock_info(addr3.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1),
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // cannot send to burnt wallet
    let info = mock_info(addr3.as_ref(), &[]);
    let env = mock_env();
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());
    let msg = ExecuteMsg::Send {
        contract: addr1,
        amount: Uint128::new(1),
        msg: send_msg,
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // cannot burn too little
    let info = mock_info(addr2.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn { remote_from: None };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::IncorrectBalance(too_little));

    // cannot burn too much
    let info = mock_info(addr3.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn { remote_from: None };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::IncorrectBalance(too_much));
}

#[test]
fn send() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let amount1 = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);
    let too_much = Uint128::from(12340321u128);
    let send_msg = Binary::from(r#"{"some":123}"#.as_bytes());

    do_instantiate(
        deps.as_mut(),
        vec![addr1.as_str(), addr2.as_str()],
        vec![amount1, Uint128::new(0)],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    // cannot send nothing
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: STAKE_ADDR.to_string(),
        amount: Uint128::zero(),
        msg: send_msg.clone(),
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // cannot send more than we have
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: addr2.to_string(),
        amount: too_much,
        msg: send_msg.clone(),
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // valid transfer to existing addr
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: addr2.to_string(),
        amount: transfer,
        msg: send_msg.clone(),
        remote_from: None,
    };
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.attributes,
        [
            ("action", "send"),
            ("from", &addr1),
            ("to", &addr2),
            ("amount", &transfer.to_string())
        ]
    );

    // ensure proper send message sent
    // this is the message we want delivered to the other side
    let binary_msg = Cw20ReceiveMsg {
        sender: addr1.clone(),
        amount: transfer,
        msg: send_msg.clone(),
    }
    .into_binary()
    .unwrap();
    // and this is how it must be wrapped for the vm to process it
    assert_eq!(
        res.messages[0],
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr2.to_string(),
            msg: binary_msg,
            funds: vec![],
        }))
    );

    // ensure balance is properly transferred
    let remainder = amount1.checked_sub(transfer).unwrap();
    assert_eq!(get_balance(deps.as_ref(), &addr1), remainder);
    assert_eq!(get_balance(deps.as_ref(), &addr2), transfer);
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        amount1
    );

    // cannot send to not a wallet
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: "not-a-wallet".to_string(),
        amount: transfer,
        msg: send_msg.clone(),
        remote_from: None,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn query_all_accounts_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // insert order and lexicographical order are different
    let acct1 = String::from("acct01");
    let acct2 = String::from("zebra");
    let acct3 = String::from("nice");
    let acct4 = String::from("aaaardvark");
    let expected_order = [acct4.clone(), acct1.clone(), acct3.clone(), acct2.clone()];

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
        Some(DAO_TUNNEL),
        None,
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
fn query_minter_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    // insert order and lexicographical order are different
    let acct1 = String::from("acct01");

    do_instantiate(
        deps.as_mut(),
        vec![acct1.as_str()],
        vec![Uint128::new(1)],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    let minter = query_minter(deps.as_ref()).unwrap();
    assert_eq!(
        minter.minters,
        Some(vec![DAO_TUNNEL.to_string(), FACTORY.to_string()])
    );
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
        Some(DAO_TUNNEL),
        None,
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.project.unwrap(), project);
    assert_eq!(marketing_info.description.unwrap(), description);
}

#[test]
fn query_download_logo_works() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    do_instantiate(deps.as_mut(), vec![], vec![], None, None, None, None);

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
        None,
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.project.unwrap(), project);
    assert_eq!(marketing_info.description.unwrap(), description);

    let project = "new-website".to_string();
    let description = "new-description".to_string();

    let res = execute_update_marketing(
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
        None,
        Some(marketing_msg),
    );

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.logo.is_none(), true);

    let logo = Logo::Url("website".to_string());

    let res = execute_upload_logo(deps.as_mut(), env, info, logo).unwrap();

    assert_eq!(res.attributes, [("action", "upload_logo")]);

    let marketing_info = query_marketing_info(deps.as_ref()).unwrap();
    assert_eq!(marketing_info.logo.is_some(), true);
}
