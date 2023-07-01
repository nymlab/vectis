import { CWClient, FactoryClient, ProxyClient } from "../../clients";
import { pluginRegInstallFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { hubDeployReportPath, hostChain, hostChainName } from "../../utils/constants";
import { CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { ProxyT, CroncatT, CronkittyT } from "../../interfaces";
import { createSingleProxyWallet } from "../../tests/mocks/proxyWallet";

//const croncat_factory_addr = "juno1mc4wfy9unvy2mwx7dskjqhh6v7qta3vqsxmkayclg4c2jude76es0jcp38";
//const tic_tac_toe = "juno1tsqeaanlxv5wlu0zdcje2lute0m8g0nszz3gwatr0tyauujskrws55h09u";

const croncat_factory_addr = "neutron1qdmeqpzlha2lgw7w90up895fu3a8p3g0gnfvd9yj04ks9z9p305qtpkxdt";
//const tic_tac_toe = "neutron1h92cjyqnh8x6t4evvsanpuh93jspjl764zdqkj72cemlcs05kdlsqkwvs5";

const plugin_id = 1;

(async function create_wallet_with_plugin() {
    const { Factory } = await import(hubDeployReportPath);
    const userClient = await CWClient.connectHostWithAccount("user");

    //// Create Vectis Account
    const factoryClient = new FactoryClient(userClient, userClient.sender, Factory);
    let walletAddr = await createSingleProxyWallet(factoryClient, "host", "juno-2429-proxy-2434-cronkitty");
    let cronkittyInstMsg = { croncat_factory_addr: croncat_factory_addr, vectis_account_addr: walletAddr };

    // Install Cronkitty on Vectis Account
    let installPlugin: ProxyT.ExecuteMsg = {
        instantiate_plugin: {
            instantiate_msg: toCosmosMsg(cronkittyInstMsg),
            label: "Cronkitty Plugin",
            plugin_params: { permissions: ["exec"] },
            src: {
                vectis_registry: plugin_id,
            },
        },
    };

    let fees = pluginRegInstallFee(hostChain).amount == "0" ? undefined : [pluginRegInstallFee(hostChain)];
    let result = await userClient.execute(userClient.sender, walletAddr, installPlugin, "auto", undefined, fees);
    let cronkittyAddr = CWClient.getContractAddrFromResult(result, "_contract_address");
    console.log(JSON.stringify(result));

    const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    console.log("Cronkitty Plugin: ", cronkittyAddr);
    console.log("Vectis Account: ", walletAddr);

    if (hostChainName != "juno_localnet") {
        //// -----------------------------------
        //// Let's create a game for this user
        //// -----------------------------------
        //const new_game_msg: CosmosMsgForEmpty = {
        //    wasm: {
        //        execute: {
        //            contract_addr: tic_tac_toe,
        //            msg: toCosmosMsg({ new_game: {} }),
        //            funds: [],
        //        },
        //    },
        //};
        //const res_new_game = await proxyClient.execute({ msgs: [new_game_msg] });
        //console.log("\n\n--> New Game: \n ", JSON.stringify(res_new_game));
        //// -----------------------------------
        //// Now controller (human) plays the first move
        //// -----------------------------------
        //const first_move: CosmosMsgForEmpty = {
        //    wasm: {
        //        execute: {
        //            contract_addr: tic_tac_toe,
        //            msg: toCosmosMsg({ play: { point: { x: 2, y: 1 } } }),
        //            funds: [],
        //        },
        //    },
        //};
        //await proxyClient.execute({ msgs: [first_move] });
        //const game_after_first_move = await userClient.queryContractSmart(tic_tac_toe, {
        //    game_info: { owner: walletAddr },
        //});
        //console.log("\n\n--> Game Status \n ", JSON.stringify(game_after_first_move));
        //// -----------------------------------
        //// Create Task on Cronkitty
        //// -----------------------------------
        //const amount_funds_for_task = "1000000";
        //const funds_in_croncat = { amount: amount_funds_for_task, denom: hostChain.feeToken };
        //let task: CronkittyT.TaskRequest = {
        //    actions: [
        //        {
        //            // I am not sure what this should be, if it is 800000 it does run but then we will need to increate the `amount_funds_for_task`
        //            gas_limit: 1000000,
        //            msg: {
        //                wasm: {
        //                    execute: {
        //                        contract_addr: tic_tac_toe,
        //                        funds: [],
        //                        // This will fail if it is not the turn of player 2, cronkitty
        //                        // or there is not yet a game
        //                        msg: toCosmosMsg({ play: {} }),
        //                    },
        //                },
        //            },
        //        },
        //    ],
        //    boundary: null,
        //    interval: { block: 5 },
        //    stop_on_fail: false,
        //};
        //let createTaskMsg: CronkittyT.ExecuteMsg = { create_task: { task: task, auto_refill: amount_funds_for_task } };
        ////const actionId = await userClient.queryContractSmart(cronkittyAddr!, { action_id: {} });
        ////let removeTaskMsg: CronkittyT.ExecuteMsg = { remove_task: { task_id: actionId - 1 } };
        //let cronKittyMsg: CosmosMsgForEmpty = {
        //    wasm: {
        //        execute: {
        //            contract_addr: cronkittyAddr!,
        //            msg: toCosmosMsg(createTaskMsg),
        //            funds: [funds_in_croncat],
        //        },
        //    },
        //};
        //const res = await proxyClient.execute({ msgs: [cronKittyMsg] });
        //console.log(JSON.stringify(res));
        //// Check Cronkitty has task
        //const actionId = await userClient.queryContractSmart(cronkittyAddr!, { action_id: {} });
        //const action = await userClient.queryContractSmart(cronkittyAddr!, { action: { action_id: actionId - 1 } });
        //console.log("Task set for Vectis Account on Cronkitty: ", JSON.stringify(action));
        //const task_addr = action.task_addr;
        //const taskRes = await userClient.queryContractSmart(task_addr, {
        //    task: { task_hash: action.task_hash },
        //});
        //console.log("Task set for Vectis Account on Croncat: ", JSON.stringify(taskRes));
    }

    //let walletAddr = "juno19xr3s04gpvfswnqv3w2grz4mtaj5farswccha9ayzlhdexcn668s5ghyg7";
    //const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    //let plugins = await proxyClient.plugins();
    //let cronkittyAddr = plugins.exec_plugins.pop();
    //const first_move: CosmosMsgForEmpty = {
    //    wasm: {
    //        execute: {
    //            contract_addr: tic_tac_toe,
    //            msg: toCosmosMsg({ play: { point: { x: 1, y: 1 } } }),
    //            funds: [],
    //        },
    //    },
    //};
    //await proxyClient.execute({ msgs: [first_move] });
    //const game_after_first_move = await userClient.queryContractSmart(tic_tac_toe, {
    //    game_info: { owner: walletAddr },
    //});
    //console.log("\n\n--> Game Status \n ", JSON.stringify(game_after_first_move));
})();
