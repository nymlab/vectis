use crate::common::common::*;
use crate::common::dao_common::*;

#[test]
fn cannot_propose_without_govec() {
    let mut suite = DaoChainSuite::init().unwrap();
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // User propose with wallet
    let propose_msg = ProposalExecuteMsg::Propose {
        title: String::from("title"),
        description: String::from("des"),
        msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: "some_addr".to_string(),
            amount: vec![coin(123, "ucosm")],
        })],
        relayed_from: None,
    };

    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &proxy_exec(&suite.proposal, &propose_msg, vec![]),
            &[],
        )
        .unwrap_err();
}

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

    suite.fast_forward_block_time(1000);

    // User propose with wallet
    let propose_msg = ProposalExecuteMsg::Propose {
        title: String::from("title"),
        description: String::from("des"),
        msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
            to_address: "some_addr".to_string(),
            amount: vec![coin(123, "ucosm")],
        })],
        relayed_from: None,
    };

    suite
        .app
        .execute_contract(
            suite.user.clone(),
            wallet_addr.clone(),
            &proxy_exec(&suite.proposal, &propose_msg, vec![]),
            &[],
        )
        .unwrap();

    let proposals = suite.query_proposals().unwrap();
    assert_eq!(proposals.proposals.len(), 1);
    let proposal = suite.query_proposal(proposals.proposals[0].id).unwrap();
    assert_eq!(proposal.proposal.title, "title");
}
