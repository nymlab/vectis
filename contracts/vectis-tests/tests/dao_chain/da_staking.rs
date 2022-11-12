use crate::common::common::*;
use crate::common::dao_common::*;

#[test]
fn with_govec_can_propose() {
    let mut suite = DaoChainSuite::init().unwrap();
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // user mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&FactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![],
    });

    // User execute proxy to claim govec
    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute {
                msgs: vec![mint_govec_msg],
            },
            &[],
        )
        .unwrap();

    let user_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(user_govec_balance.balance, Uint128::from(2u8));

    let stake_msg = GovecExecuteMsg::Send {
        contract: suite.cw20_stake.to_string(),
        amount: Uint128::one(),
        msg: to_binary(&ReceiveMsg::Stake {}).unwrap(),
        relayed_from: None,
    };

    // User stakes wallet govec
    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &proxy_exec(&suite.govec, &stake_msg, vec![]),
            &[],
        )
        .unwrap();

    suite.app.update_block(|b| b.height += 10);
    let balance = suite
        .query_staked_balance_at_height(wallet_addr.to_string(), None)
        .unwrap();
    assert_eq!(balance.balance, Uint128::one());

    // User unstakes wallet govec
    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &proxy_exec(
                &suite.cw20_stake,
                &StakeExecuteMsg::Unstake {
                    amount: Uint128::one(),
                    relayed_from: None,
                },
                vec![],
            ),
            &[],
        )
        .unwrap();

    suite.app.update_block(|b| b.height += 10);
    let balance = suite
        .query_staked_balance_at_height(wallet_addr.to_string(), None)
        .unwrap();
    assert_eq!(balance.balance, Uint128::zero());

    // User claim wallet govec
    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &proxy_exec(
                &suite.cw20_stake,
                &StakeExecuteMsg::Claim { relayed_from: None },
                vec![],
            ),
            &[],
        )
        .unwrap();
    let user_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(user_govec_balance.balance, Uint128::from(2u8));
}
