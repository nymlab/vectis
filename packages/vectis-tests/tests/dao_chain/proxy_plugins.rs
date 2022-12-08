use cosmwasm_std::{coin, BankMsg, Coin, CosmosMsg};
use cronkitty::contract::StoredMsgsResp;
use cw_croncat_core::{
    msg::{
        AgentResponse, ExecuteMsg as CCExecMsg, InstantiateMsg as CCInstantMsg,
        QueryMsg as CCQueryMsg, TaskRequest, TaskResponse,
    },
    types::{Action, Interval},
};
use cw_multi_test::Executor;
use cw_rules_core::msg::InstantiateMsg as CRInstMsg;

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
    QueryMsg as CronKittyQueryMsg,
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

// These addresses need to be well formed as balances are queried in croncat contract
const AGENT_BENEFICIARY: &str = "wasm1ucl9dulgww2trng0dmunj348vxneufu5nk4yy4";
const AGENT0: &str = "wasm1ucl9dulgww2trng0dmunj348vxneufu5n11yy4";

// TODO: add registry as cronkitty is trusted
//
//  This is a full cycle integration test with
//  - croncat contracts (core + rules)
//  - proxy
//  - cronkitty contract
//
//  Test is what a user might go through
//  - create task (once? )
//  - agent executes task via croncat -> cronkitty -> proxy
//  - refill task
//  - remote task (get refund)

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

    // ==============================================================
    // Instantiate Croncat and add Agent to execute tasks
    // ==============================================================
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

    // quick agent register
    suite
        .app
        .send_tokens(
            suite.deployer.clone(),
            Addr::unchecked(AGENT0),
            &[coin(10_000_00, "ucosm")],
        )
        .unwrap();

    let msg = CCExecMsg::RegisterAgent {
        payable_account_id: Some(AGENT_BENEFICIARY.to_string()),
    };
    suite
        .app
        .execute_contract(Addr::unchecked(AGENT0), croncat.clone(), &msg, &[])
        .unwrap();
    //app.execute_contract(
    //    Addr::unchecked(contract_addr.clone()),
    //    contract_addr.clone(),
    //    &msg,
    //    &[],
    //)
    //.unwrap();

    // This fast forwards 10 blocks and arg is timestamp
    suite.fast_forward_block_time(10000);

    // ==============================================================
    // Instantiate CronKitty
    // ==============================================================
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

    // ==============================================================
    // Create Task on Cronkitty + Croncat
    // ==============================================================

    let to_send_amount = 500;
    suite
        .app
        .send_tokens(
            suite.deployer.clone(),
            wallet_address.clone(),
            &[coin(to_send_amount, "ucosm")],
        )
        .unwrap();

    let init_proxy_balance = suite.query_balance(&wallet_address).unwrap();

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: suite.dao.to_string(),
        amount: vec![coin(to_send_amount, "ucosm")],
    });
    let tq = TaskRequest {
        interval: Interval::Block(5),
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: msg.clone(),
            gas_limit: Some(150_000),
        }],
        rules: None,
        cw20_coins: vec![],
    };

    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_address.clone(),
            &proxy_exec(
                &cronkitty,
                &CronKittyExecMsg::CreateTask { tq },
                vec![coin(150_000, "ucosm")],
            ),
            // to send exact amount needed to the proxy so balance doesnt change
            &[coin(150_000, "ucosm")],
        )
        .unwrap();

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

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].owner_id, cronkitty);

    let task_on_cronkitty: StoredMsgsResp = suite
        .app
        .wrap()
        .query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: cronkitty.to_string(),
            msg: to_binary(&CronKittyQueryMsg::Action { action_id: 0 }).unwrap(),
        }))
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].owner_id, cronkitty);
    assert_eq!(tasks[0].task_hash, task_on_cronkitty.task_hash.unwrap());

    // This fast forwards 10 blocks and arg is timestamp
    suite.fast_forward_block_time(10000);
    // ==============================================================
    // Agent executes proxy call
    // ==============================================================

    // There is only one task in the queue
    let proxy_call_msg = CCExecMsg::ProxyCall { task_hash: None };
    suite
        .app
        .execute_contract(
            Addr::unchecked(AGENT0),
            croncat.clone(),
            &proxy_call_msg,
            &vec![],
        )
        .unwrap();

    let agent: Option<AgentResponse> = suite
        .app
        .wrap()
        .query_wasm_smart(
            croncat.clone(),
            &CCQueryMsg::GetAgent {
                account_id: String::from(AGENT0),
            },
        )
        .unwrap();

    assert_eq!(agent.unwrap().total_tasks_executed, 1);

    let after_proxy_balance = suite.query_balance(&wallet_address).unwrap();

    // Ensure it happened
    assert_eq!(
        init_proxy_balance.amount - after_proxy_balance.amount,
        Uint128::from(to_send_amount)
    );
    // ==============================================================
    // Proxy refill task from cronkitty and croncat
    // ==============================================================

    let before_refill_tasks: Vec<TaskResponse> = suite
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

    let refill_amount = 150_000;
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_address.clone(),
            &proxy_exec(
                &cronkitty,
                &CronKittyExecMsg::RefillTask { task_id: 0 },
                vec![coin(refill_amount, "ucosm")],
            ),
            // to send exact amount needed to the proxy so balance doesnt change
            &[coin(refill_amount, "ucosm")],
        )
        .unwrap();
    suite.fast_forward_block_time(10000);

    let after_refill_tasks: Vec<TaskResponse> = suite
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

    assert_eq!(
        after_refill_tasks[0].total_deposit[0].amount
            - before_refill_tasks[0].total_deposit[0].amount,
        Uint128::from(refill_amount)
    );

    // ==============================================================
    // Proxy remove task from cronkitty and croncat
    // ==============================================================
    suite
        .app
        .execute_contract(
            suite.controller.clone(),
            wallet_address.clone(),
            &proxy_exec(
                &cronkitty,
                &CronKittyExecMsg::RemoveTask { task_id: 0 },
                vec![],
            ),
            // to send exact amount needed to the proxy so balance doesnt change
            &[],
        )
        .unwrap();
    suite.fast_forward_block_time(10000);

    let after_remove_task: Vec<TaskResponse> = suite
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

    assert!(after_remove_task.is_empty());

    let task_on_cronkitty: Result<StoredMsgsResp, StdError> =
        suite
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: cronkitty.to_string(),
                msg: to_binary(&CronKittyQueryMsg::Action { action_id: 0 }).unwrap(),
            }));

    task_on_cronkitty.unwrap_err();
}
