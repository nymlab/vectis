use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
};
use cosmwasm_std::{
    coins, from_binary, Binary, CosmosMsg, Deps, DepsMut, StdError, SubMsg, Uint128, WasmMsg,
};

use crate::contract::*;
use crate::enumerable::*;
use crate::error::*;
use crate::msg::*;
use crate::state::MinterData;

use cw20::{BalanceResponse, Cw20Coin, Cw20ReceiveMsg, TokenInfoResponse};

fn get_balance<T: Into<String>>(deps: Deps, address: T) -> Uint128 {
    query_balance(deps, address.into()).unwrap().balance
}

const STAKE_ADDR: &str = "staker";
const MINTER_ADDR: &str = "factory";
const DAO_ADDR: &str = "dao";

// this will set up the instantiation for other tests
fn do_instantiate(
    mut deps: DepsMut,
    addr: Vec<&str>,
    amount: Vec<Uint128>,
    minter: &str,
    cap: Option<Uint128>,
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
    let mint = MinterData {
        minter: minter.to_string(),
        cap,
    };
    let instantiate_msg = InstantiateMsg {
        name: "Auto Gen".to_string(),
        symbol: "AUTO".to_string(),
        initial_balances: coins,
        staking_addr: None,
        minter: Some(mint.clone()),
        marketing: None,
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
    assert_eq!(
        query_minter(deps.as_ref()).unwrap(),
        Some(MinterData {
            minter: mint.minter,
            cap: mint.cap
        })
    );
    assert_eq!(query_dao(deps.as_ref()).unwrap(), DAO_ADDR);
    meta
}

#[test]
fn mintable() {
    let mut deps = mock_dependencies();
    let amount = Uint128::new(11223344);
    let minter = String::from("asmodat");
    let limit = Uint128::new(511223344);
    let instantiate_msg = InstantiateMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        initial_balances: vec![Cw20Coin {
            address: "addr0000".into(),
            amount,
        }],
        minter: Some(MinterData {
            minter: minter.clone(),
            cap: Some(limit),
        }),
        staking_addr: None,
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
            total_supply: amount,
        }
    );
    assert_eq!(
        get_balance(deps.as_ref(), "addr0000"),
        Uint128::new(11223344)
    );
    assert_eq!(
        query_minter(deps.as_ref()).unwrap(),
        Some(MinterData {
            minter,
            cap: Some(limit),
        }),
    );
}

#[test]
fn cannot_mint_over_cap() {
    let mut deps = mock_dependencies();
    let amount = Uint128::new(11223344);
    let minter = String::from("asmodat");
    let limit = Uint128::new(11223300);
    let instantiate_msg = InstantiateMsg {
        name: "Cash Token".to_string(),
        symbol: "CASH".to_string(),
        initial_balances: vec![Cw20Coin {
            address: String::from("addr0000"),
            amount,
        }],
        minter: Some(MinterData {
            minter,
            cap: Some(limit),
        }),
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
        DAO_ADDR,
        Some(limit),
    );

    let msg = ExecuteMsg::UpdateStakingAddr {
        new_addr: new_staking.clone(),
    };

    // only dao can update staking
    let info = mock_info(MINTER_ADDR, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // dao can update staking
    let msg = ExecuteMsg::UpdateStakingAddr {
        new_addr: new_staking.clone(),
    };
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(query_staking(deps.as_ref()).unwrap(), new_staking);
}

#[test]
fn dao_can_update_dao_addr() {
    let mut deps = mock_dependencies();
    let genesis = String::from("genesis");
    let amount = Uint128::new(0);
    let limit = Uint128::new(12);

    let new_dao = String::from("new_dao");

    do_instantiate(
        deps.as_mut(),
        vec![genesis.as_str()],
        vec![amount],
        MINTER_ADDR,
        Some(limit),
    );

    let msg = ExecuteMsg::UpdateDaoAddr {
        new_addr: new_dao.clone(),
    };

    // only dao can update staking
    let info = mock_info(MINTER_ADDR, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg.clone()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(query_dao(deps.as_ref()).unwrap(), new_dao);
}

#[test]
fn dao_can_update_mint_data() {
    let mut deps = mock_dependencies();
    let genesis = String::from("genesis");
    let amount = Uint128::new(0);
    let limit = Uint128::new(12);

    do_instantiate(
        deps.as_mut(),
        vec![genesis.as_str()],
        vec![amount],
        MINTER_ADDR,
        Some(limit),
    );

    let msg = ExecuteMsg::UpdateMintData { new_mint: None };

    // only dao can update mint data
    let info = mock_info(MINTER_ADDR, &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    let new_mint = MinterData {
        minter: "new_minter".to_string(),
        cap: Some(Uint128::new(200)),
    };
    // dao can update mint data
    let msg = ExecuteMsg::UpdateMintData {
        new_mint: Some(new_mint.clone()),
    };
    let info = mock_info(DAO_ADDR, &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(
        query_minter(deps.as_ref()).unwrap().unwrap().minter,
        new_mint.minter
    );
    assert_eq!(
        query_minter(deps.as_ref()).unwrap().unwrap().cap,
        new_mint.cap
    );
}

#[test]
fn can_mint_by_minter() {
    let mut deps = mock_dependencies();

    let genesis = String::from("genesis");
    let amount = Uint128::new(0);
    let minter = String::from("asmodat");
    let limit = Uint128::new(1);
    do_instantiate(
        deps.as_mut(),
        vec![genesis.as_str()],
        vec![amount],
        &minter,
        Some(limit),
    );

    // minter can mint coins to some winner
    let winner = String::from("lucky");
    let msg = ExecuteMsg::Mint {
        new_wallet: winner.clone(),
    };

    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    assert_eq!(get_balance(deps.as_ref(), genesis), amount);
    assert_eq!(get_balance(deps.as_ref(), winner.clone()), Uint128::new(1));

    // but if it exceeds cap, it fails cap is enforced
    let msg = ExecuteMsg::Mint { new_wallet: winner };
    let info = mock_info(minter.as_ref(), &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::CannotExceedCap {});
}

#[test]
fn others_cannot_mint() {
    let mut deps = mock_dependencies();
    do_instantiate(
        deps.as_mut(),
        vec!["genesis"],
        vec![Uint128::new(1234)],
        MINTER_ADDR,
        None,
    );

    let msg = ExecuteMsg::Mint {
        new_wallet: String::from("genesis"),
    };
    let info = mock_info("anyone else", &[]);
    let env = mock_env();
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn instantiate_multiple_accounts() {
    let mut deps = mock_dependencies();
    let amount1 = Uint128::from(11223344u128);
    let addr1 = String::from("addr0001");
    let amount2 = Uint128::from(7890987u128);
    let addr2 = String::from("addr0002");
    let instantiate_msg = InstantiateMsg {
        name: "Bash Shell".to_string(),
        symbol: "BASH".to_string(),
        initial_balances: vec![
            Cw20Coin {
                address: addr1.clone(),
                amount: amount1,
            },
            Cw20Coin {
                address: addr2.clone(),
                amount: amount2,
            },
        ],
        minter: None,
        staking_addr: None,
        marketing: None,
    };
    let info = mock_info("creator", &[]);
    let env = mock_env();
    let res = instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());

    assert_eq!(
        query_token_info(deps.as_ref()).unwrap(),
        TokenInfoResponse {
            name: "Bash Shell".to_string(),
            symbol: "BASH".to_string(),
            decimals: 0,
            total_supply: amount1 + amount2,
        }
    );
    assert_eq!(get_balance(deps.as_ref(), addr1), amount1);
    assert_eq!(get_balance(deps.as_ref(), addr2), amount2);
}

#[test]
fn queries_work() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let amount1 = Uint128::from(12340000u128);

    let expected = do_instantiate(
        deps.as_mut(),
        vec![addr1.as_str()],
        vec![amount1],
        MINTER_ADDR,
        None,
    );

    // check meta query
    let loaded = query_token_info(deps.as_ref()).unwrap();
    assert_eq!(expected, loaded);

    let _info = mock_info(&addr1, &[]);
    let env = mock_env();
    // check balance query (full)
    let data = query(
        deps.as_ref(),
        env.clone(),
        QueryMsg::Balance { address: addr1 },
    )
    .unwrap();
    let loaded: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(loaded.balance, amount1);

    // check balance query (empty)
    let data = query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: String::from("addr0002-not-a-wallet"),
        },
    );
    assert!(data.is_err());
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
        MINTER_ADDR,
        None,
    );

    // cannot transfer nothing
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: Uint128::zero(),
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::InvalidZeroAmount {});

    // cannot send more than we have
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr2.clone(),
        amount: too_much,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send from empty account
    let info = mock_info(not_wallet.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: transfer,
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // cannot send to non-existing accounts
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: not_wallet.clone(),
        amount: transfer,
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
        MINTER_ADDR,
        None,
    );

    let initial_total_supply = query_token_info(deps.as_ref()).unwrap().total_supply;

    // valid burn reduces total supply and remove account from BALANCES
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {};
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let remainder = initial_total_supply.checked_sub(right_amount).unwrap();
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        remainder
    );
    assert!(query(
        deps.as_ref(),
        env,
        QueryMsg::Balance {
            address: addr1.clone(),
        },
    )
    .is_err());

    // cannot transfer to burnt wallet
    let info = mock_info(addr3.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: addr1.clone(),
        amount: Uint128::new(1),
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
    };
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // cannot burn too little
    let info = mock_info(addr2.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {};
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::IncorrectBalance(too_little));

    // cannot burn too much
    let info = mock_info(addr3.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {};
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
        MINTER_ADDR,
        None,
    );

    // cannot send nothing
    let info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: STAKE_ADDR.to_string(),
        amount: Uint128::zero(),
        msg: send_msg.clone(),
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
        MINTER_ADDR,
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
        MINTER_ADDR,
        None,
    );

    let minter = query_minter(deps.as_ref()).unwrap();
    assert_eq!(minter.unwrap().minter, MINTER_ADDR);
}
