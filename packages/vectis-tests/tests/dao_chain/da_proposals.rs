use crate::common::common::*;
use crate::common::dao_common::*;

fn get_pre_proposal_msg() -> PrePropExecMsg {
    PrePropExecMsg::Propose {
        msg: ProposeMessage::Propose {
            title: String::from("title"),
            description: String::from("des"),
            msgs: vec![CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: "some_addr".to_string(),
                amount: vec![coin(123, "ucosm")],
            })],
            relayed_from: None,
        },
    }
}

#[test]
fn cannot_pre_propose_without_govec() {
    let mut suite = DaoChainSuite::init().unwrap();
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // Controller propose with wallet
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr,
            &proxy_exec(&suite.pre_prop, &get_pre_proposal_msg(), vec![]),
            &[],
        )
        .unwrap_err();
}

#[test]
fn with_govec_can_pre_propose_and_approver_can_approve() {
    let mut suite = DaoChainSuite::init().unwrap();
    // Create a new wallet
    let wallet_addr = suite
        .create_new_proxy(suite.controller.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    // controller mint govec
    let mint_govec_msg = CosmosMsg::<()>::Wasm(WasmMsg::Execute {
        contract_addr: suite.factory.to_string(),
        msg: to_binary(&FactoryExecuteMsg::ClaimGovec {}).unwrap(),
        funds: vec![coin(CLAIM_FEE, "ucosm")],
    });

    // Controller execute proxy to claim govec
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute {
                msgs: vec![mint_govec_msg],
            },
            &[coin(CLAIM_FEE, "ucosm")],
        )
        .unwrap();

    let controller_govec_balance = suite.query_govec_balance(&wallet_addr).unwrap();
    assert_eq!(controller_govec_balance.balance, Uint128::from(2u8));
    let stake_msg = GovecExecuteMsg::Send {
        contract: suite.cw20_stake.to_string(),
        amount: Uint128::one(),
        msg: to_binary(&ReceiveMsg::Stake {}).unwrap(),
        relayed_from: None,
    };

    // Controller stakes wallet govec
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr.clone(),
            &proxy_exec(&suite.govec, &stake_msg, vec![]),
            &[],
        )
        .unwrap();

    suite.fast_forward_block_time(1000);

    let pre_proposals = suite.query_pre_proposals().unwrap();
    assert_eq!(pre_proposals.len(), 0);

    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_addr,
            &proxy_exec(&suite.pre_prop, &get_pre_proposal_msg(), vec![]),
            &[],
        )
        .unwrap();

    let pre_proposals = suite.query_pre_proposals().unwrap();
    assert_eq!(pre_proposals.len(), 1);
    let id = pre_proposals[0].approval_id;

    // No Proposal yet
    let proposals = suite.query_proposals().unwrap();
    assert_eq!(proposals.proposals.len(), 0);

    // Approver can approve pre_propose
    suite
        .app
        .execute_contract(
            suite.prop_approver.clone(),
            suite.pre_prop.clone(),
            &PrePropExecMsg::Extension {
                msg: PrePropExecExt::Approve { id },
            },
            &[],
        )
        .unwrap();

    // TODO check deposit
    let proposals = suite.query_proposals().unwrap();
    assert_eq!(proposals.proposals.len(), 1);
    let pre_proposals = suite.query_pre_proposals().unwrap();
    assert_eq!(pre_proposals.len(), 0);
}
