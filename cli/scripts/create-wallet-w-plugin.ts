import { CWClient, FactoryClient, ProxyClient } from "../clients";
import { pluginRegInstallFee } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import { daoDeployReportPath, hostChain, hostChainName } from "../utils/constants";
import { Vote, ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../interfaces/Cw3Flex.types";
import { FactoryT, ProxyT, CroncatT } from "../interfaces";
import * as accounts from "../config/accounts";
import { createSingleProxyWallet } from "../tests/mocks/proxyWallet";

const croncat_factory_addr = "juno16ze0ve5q5z0wd4n5yp2kayeqn5el0tzklpafj7zjjchfh93x4wfsa8fxur";
const code_id = 1002;

(async function create_wallet_with_plugin() {
    const { Factory, PluginCommittee, PluginRegistry } = await import(daoDeployReportPath);

    const userClient = await CWClient.connectHostWithAccount("user");
    const factoryClient = new FactoryClient(userClient, userClient.sender, Factory);

    //// Create Vectis Account
    //let walletAddr = await createSingleProxyWallet(factoryClient, "host");
    //let cronkittyInstMsg = { croncat_factory_addr: croncat_factory_addr };

    //// Install Cronkitty on Vectis Account
    //let installPlugin: ProxyT.ExecuteMsg = {
    //    instantiate_plugin: {
    //        instantiate_msg: toCosmosMsg(cronkittyInstMsg),
    //        label: "Cronkitty Plugin",
    //        plugin_params: { grantor: false },
    //        src: {
    //            code_id: code_id,
    //        },
    //    },
    //};

    //await userClient.execute(userClient.sender, walletAddr, installPlugin, "auto", undefined, [
    //    pluginRegInstallFee(hostChain),
    //]);

    //const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    //let plugins = await proxyClient.plugins({});
    //let cronkittyAddr = plugins.plugins.pop();

    //console.log("WalletAddr: ", walletAddr);
    //console.log("PluginAddr: ", cronkittyAddr);

    const walletAddr = "juno1nzqrqemjsyhr9mz9s0ge5fjslyc3wkr7s4v2ggcl2evz4taljapsq3vc9h";
    const cronkittyAddr = "juno1mup8qelkmhz5vnpyd7cku3jlhax7hysvtl4vk4z7e5kv7f9n08qq8g42hy";
    const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    let plugins = await proxyClient.plugins({});

    // Create Task on Cronkitty
    // This sends 1 juno to wallet itself every hour
    const funds = { amount: (1000000).toString(), denom: hostChain.feeToken };
    const gas = { amount: (100000).toString(), denom: hostChain.feeToken };
    let task: CroncatT.TaskRequest = {
        actions: [
            {
                gas_limit: 100000,
                msg: {
                    bank: {
                        send: {
                            amount: [funds],
                            to_address: walletAddr,
                        },
                    },
                },
            },
        ],
        boundary: { height: { end: "632926" } },
        interval: { block: 20 },
        stop_on_fail: false,
    };

    let createTaskMsg = { create_task: { task: task } };

    let cronKittyMsg: CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: cronkittyAddr!,
                funds: [gas],
                msg: toCosmosMsg(createTaskMsg),
            },
        },
    };

    const res = await proxyClient.execute({ msgs: [cronKittyMsg] });
    console.log(JSON.stringify(res));

    // Check Cronkitty has task
    const actionId = await userClient.queryContractSmart(cronkittyAddr!, { action_id: {} });
    console.log("actionID: ", actionId);
    const action = await userClient.queryContractSmart(cronkittyAddr!, { action: { action_id: actionId - 1 } });

    console.log("Task set for Vectis Account: ", JSON.stringify(action));

    writeInCacheFolder(`registeredPlugins.json`, JSON.stringify({ ...plugins, walletAddr }, null, 2));
})();
