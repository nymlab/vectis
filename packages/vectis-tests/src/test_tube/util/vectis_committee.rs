use super::contract::Contract;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, WasmMsg};
use cw3::ProposalListResponse;
use cw3_flex_multisig::msg::{ExecuteMsg as cw3flexExecMsg, QueryMsg as cw3flexQueryMsg};
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContractResponse;
use osmosis_test_tube::OsmosisTestApp;
use serde::Serialize;
use test_tube::{RunnerExecuteResult, SigningAccount};

pub fn execute<'a, T>(
    app: &OsmosisTestApp,
    deployer_addr: String,
    targe_contract: String,
    exec_msg: &T,
    funds: &'a [Coin],
    signer: &SigningAccount,
) -> RunnerExecuteResult<MsgExecuteContractResponse>
where
    T: Serialize + ?Sized,
{
    let deployer = Contract::from_addr(&app, deployer_addr.clone());

    deployer
        .execute(&execute_msg(targe_contract, exec_msg, funds), funds, signer)
        .unwrap();

    let props: ProposalListResponse = deployer
        .query(&cw3flexQueryMsg::ReverseProposals {
            start_before: None,
            limit: None,
        })
        .unwrap();

    let id = props.proposals[0].id;

    deployer.execute(&cw3flexExecMsg::Execute { proposal_id: id }, &[], signer)
}

fn execute_msg<'a, T>(
    target_contract: String,
    exec_msg: &T,
    exec_msg_fund: &'a [Coin],
) -> cw3flexExecMsg
where
    T: Serialize + ?Sized,
{
    cw3flexExecMsg::Propose {
        title: "exec".into(),
        description: "exec".to_string(),
        msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: target_contract,
            msg: to_binary(&exec_msg).unwrap(),
            funds: exec_msg_fund.to_vec(),
        })],
        latest: None,
    }
}
