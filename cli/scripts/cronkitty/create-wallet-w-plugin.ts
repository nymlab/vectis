import { CWClient, FactoryClient, ProxyClient } from "../../clients";
import { pluginRegInstallFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { writeInCacheFolder } from "../../utils/fs";
import { hubDeployReportPath, hostChain, hostChainName } from "../../utils/constants";
import { Vote, ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { FactoryT, ProxyT, CroncatT } from "../../interfaces";
import * as accounts from "../../config/accounts";
import { createSingleProxyWallet } from "../../tests/mocks/proxyWallet";

const croncat_factory_addr = "juno124vcmqsukhmuy6psm45a2tdg5354rnemdetqhjt72ynju666gf0qpxmlxz";
const plugin_id = 1;

(async function create_wallet_with_plugin() {
    const { Factory, PluginRegistry } = await import(hubDeployReportPath);

    const userClient = await CWClient.connectHostWithAccount("user");

    //// Create Vectis Account
    const factoryClient = new FactoryClient(userClient, userClient.sender, Factory);
    let walletAddr = await createSingleProxyWallet(factoryClient, "host");

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

    const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    let plugins = await proxyClient.plugins();
    let cronkittyAddr = plugins.exec_plugins.pop();
    console.log("Vectis Account: ", walletAddr);
    console.log("CronKitty Addr: ", cronkittyAddr);

    let ipfs_hash = await userClient.queryContractSmart(PluginRegistry, {
        query_metadata_link: { contract_addr: cronkittyAddr },
    });
    console.log("ipfs_hash: ", ipfs_hash);

    if (hostChainName != "juno_localnet") {
        // Create Task on Cronkitty
        // This sends 1 juno to wallet itself every
        const funds = { amount: (1000000).toString(), denom: hostChain.feeToken };
        const gas = { amount: (100000).toString(), denom: hostChain.feeToken };
        let task: CroncatT.TaskRequest = {
            actions: [
                {
                    gas_limit: 100000,
                    msg: {
                        bank: {
                            send: {
                                amount: [],
                                to_address: walletAddr,
                            },
                        },
                    },
                },
            ],
            boundary: { height: { end: "1093101" } },
            interval: { block: 500 },
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
    }

    writeInCacheFolder(`registeredPlugins.json`, JSON.stringify({ ...plugins, walletAddr }, null, 2));
})();
