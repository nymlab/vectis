use crate::tests::*;

#[test]
fn remote_relayed_transfer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let remote_addr = String::from("remote_addr0001");
    let dao_addr = String::from("dao_addr0002");
    let not_wallet = String::from("not_wallet");
    let amount1 = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);

    do_instantiate(
        deps.as_mut(),
        vec![remote_addr.as_str(), dao_addr.as_str()],
        vec![amount1, Uint128::zero()],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );

    // dao_tunnel can relay transfer
    let info = mock_info(DAO_TUNNEL, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Transfer {
        recipient: dao_addr.clone(),
        amount: transfer,
        relayed_from: Some(remote_addr.clone()),
    };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        res.attributes,
        [
            ("action", "transfer"),
            ("from", &remote_addr),
            ("to", &dao_addr),
            ("amount", &transfer.to_string())
        ]
    );

    assert_eq!(get_balance(deps.as_ref(), dao_addr.clone()), transfer);
    assert_eq!(
        get_balance(deps.as_ref(), remote_addr.clone()),
        amount1.saturating_sub(transfer)
    );
    assert_eq!(
        query_token_info(deps.as_ref()).unwrap().total_supply,
        amount1
    );

    // dao_tunnel cannot relay message from non existing wallets
    let failing_msg = ExecuteMsg::Transfer {
        recipient: dao_addr.clone(),
        amount: transfer,
        relayed_from: Some(not_wallet.clone()),
    };
    let err = execute(deps.as_mut(), env.clone(), info, failing_msg).unwrap_err();
    assert!(matches!(err, ContractError::Std(StdError::Overflow { .. })));

    // not dao tunnel cannot relay transfers
    let failing_msg = ExecuteMsg::Transfer {
        recipient: dao_addr.clone(),
        amount: transfer,
        relayed_from: Some(not_wallet.clone()),
    };
    let info_wrong_dao_tunnel = mock_info("not_dao_tunnel", &[]);
    let err = execute(deps.as_mut(), env, info_wrong_dao_tunnel, failing_msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn burn() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let remote_addr = String::from("remote_addr0001");
    let dao_addr = String::from("dao_addr0002");

    do_instantiate(
        deps.as_mut(),
        vec![remote_addr.as_str(), dao_addr.as_str()],
        vec![Uint128::one(), Uint128::zero()],
        Some(FACTORY),
        Some(DAO_TUNNEL),
        None,
        None,
    );
    let initial_total_supply = query_token_info(deps.as_ref()).unwrap().total_supply;

    // valid burn update from dao_tunnel reduces total supply and remove account from BALANCES
    let info = mock_info(DAO_TUNNEL.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Burn {
        relayed_from: Some(remote_addr.clone()),
    };
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
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
            address: remote_addr.clone(),
        },
    )
    .unwrap();
    let balance: BalanceResponse = from_binary(&data).unwrap();
    assert_eq!(balance.balance, Uint128::new(0));

    // invalid dao_tunnel cannot burn token
    let failing_info = mock_info("not_dao_tunnel", &[]);
    let err = execute(deps.as_mut(), env, failing_info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn send() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let addr1 = String::from("addr0001");
    let addr2 = String::from("addr0002");
    let amount1 = Uint128::from(12340000u128);
    let transfer = Uint128::from(76543u128);
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

    // valid transfer to existing addr
    let info = mock_info(DAO_TUNNEL, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: addr2.to_string(),
        amount: transfer,
        msg: send_msg.clone(),
        relayed_from: Some(addr1.clone()),
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

    // not dao_tunnel sender cannot relay send
    let failing_info = mock_info(addr1.as_ref(), &[]);
    let env = mock_env();
    let msg = ExecuteMsg::Send {
        contract: addr2,
        amount: transfer,
        msg: send_msg.clone(),
        relayed_from: Some(addr1.clone()),
    };
    let err = execute(deps.as_mut(), env, failing_info, msg).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});
}
