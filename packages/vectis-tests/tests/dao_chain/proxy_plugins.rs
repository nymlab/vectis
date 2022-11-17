use cosmwasm_std::{coin, BankMsg, Coin, CosmosMsg};
use cw_croncat_core::{
    msg::{
        ExecuteMsg as CCExecMsg, InstantiateMsg as CCInstantMsg, QueryMsg as CCQueryMsg,
        TaskRequest, TaskResponse,
    },
    types::{Action, Interval},
};
use cw_multi_test::Executor;
use cw_rules_core::msg::InstantiateMsg as CRInstMsg;
use vectis_proxy::{msg::ExecuteMsg as ProxyExecuteMsg, msg::PluginParams};

pub use cw_croncat::entry::{
    execute as croncat_execute, instantiate as croncat_instantiate, query as croncat_query,
    reply as croncat_reply,
};

pub use cw_rules::contract::{
    execute as croncat_rules_execute, instantiate as croncat_rules_instantiate,
    query as croncat_rules_query,
};

pub use cronkitty::contract::{
    CronKittyPlugin, ExecMsg as CronKittyExecMsg, InstantiateMsg as CronKittyInstMsg,
};

use crate::common::{common::*, dao_common::*};

pub fn contract_croncat() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(croncat_execute, croncat_instantiate, croncat_query)
        .with_reply(croncat_reply);
    Box::new(contract)
}

pub fn contract_croncat_rules() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        croncat_rules_execute,
        croncat_rules_instantiate,
        croncat_rules_query,
    );
    Box::new(contract)
}

#[test]
fn cronkitty_plugin_works() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    // Instantiate CronCat
    let croncat_code_id = suite.app.store_code(contract_croncat());
    let croncat_rules_code_id = suite.app.store_code(contract_croncat_rules());

    let croncat_rules = suite
        .app
        .instantiate_contract(
            croncat_rules_code_id,
            suite.deployer.clone(),
            &CRInstMsg {},
            &[],
            "Croncat_rules",
            None,
        )
        .unwrap();

    let inst_msg = CCInstantMsg {
        denom: "ucosm".into(),
        cw_rules_addr: croncat_rules.into(),
        owner_id: Some(suite.deployer.clone().into()),
        gas_base_fee: None,
        gas_action_fee: None,
        gas_fraction: None,
        agent_nomination_duration: None,
    };

    let croncat = suite
        .app
        .instantiate_contract(
            croncat_code_id,
            suite.deployer.clone(),
            &inst_msg,
            &[],
            "Croncat",
            None,
        )
        .unwrap();

    // Instantiate CronKitty
    let cronkitty_code_id = suite.app.store_code(Box::new(CronKittyPlugin::new()));
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_address.clone(),
            &ProxyExecuteMsg::InstantiatePlugin::<Empty> {
                code_id: cronkitty_code_id,
                instantiate_msg: to_binary(&CronKittyInstMsg {
                    croncat_addr: croncat.to_string(),
                    denom: "ucosm".into(),
                })
                .unwrap(),
                plugin_params: PluginParams { grantor: false },
                label: "cronkitty-plugin".into(),
            },
            &[coin(10000, "ucosm")],
        )
        .unwrap();

    let cronkitty = suite.query_plugins(&wallet_address).unwrap().plugins[0].clone();

    // Create Task on Cronkitty + Croncat
    let to_send_amount = 111;
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: suite.dao.to_string(),
        amount: vec![coin(to_send_amount, "ucosm")],
    });
    let tq = TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: Some(150_000),
        }],
        rules: None,
        cw20_coins: vec![],
    };

    let res = suite
        .app
        .execute_contract(
            suite.controller,
            wallet_address.clone(),
            &proxy_exec(
                &cronkitty,
                &CronKittyExecMsg::CreateTask { tq },
                vec![coin(150_000, "ucosm")],
            ),
            // to top up the proxy wallet
            &[coin(200_000, "ucosm")],
        )
        .unwrap();

    println!(" EVENTS: {:?}", res);

    // Check task is added on Croncat
    let tasks: Vec<TaskResponse> = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: croncat.to_string(),
            msg: to_binary(&CCQueryMsg::GetTasks {
                from_index: None,
                limit: None,
            })
            .unwrap(),
        }))
        .unwrap();

    // remove with
    // tasks[0].taskhash

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].owner_id, cronkitty);
}
