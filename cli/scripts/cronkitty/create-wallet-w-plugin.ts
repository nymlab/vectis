import { CWClient, FactoryClient, ProxyClient } from "../../clients";
import { pluginRegInstallFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { writeInCacheFolder } from "../../utils/fs";
import { hubDeployReportPath, hostChain, hostChainName } from "../../utils/constants";
import { Vote, ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { FactoryT, ProxyT, CroncatT } from "../../interfaces";
import * as accounts from "../../config/accounts";
import { createSingleProxyWallet } from "../../tests/mocks/proxyWallet";

const croncat_factory_addr = "juno1n7gsa2zf2qsa0rl526pqc6v2ljq45qw5df9tfm26fdm76tupv0fq38xpan";
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

    //let walletAddr = "juno1g9t0avpeudadfaz9fafe6gzjuw0rltpfd2hzvmn5armwnt39smvquc0lnz";
    const proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
    let plugins = await proxyClient.plugins();
    let cronkittyAddr = plugins.exec_plugins.pop();
    console.log("Vectis Account: ", walletAddr);
    console.log("CronKitty Addr: ", cronkittyAddr);
    //let cronkittyAddr = "juno1jrtzgl5emvc35ds7knu6jtsa65g9hd6ymexjafmer4np22x4304qzg8ued";

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
                                amount: [funds],
                                to_address: walletAddr,
                            },
                        },
                    },
                },
            ],
            boundary: { height: { end: "1684727" } },
            interval: { block: 100 },
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

        console.log("Task set for Vectis Account on Cronkitty: ", JSON.stringify(action));
        const task_addr = action.tash_addr;
        const taskRes = await userClient.queryContractSmart(task_addr, {
            current_task: {},
        });

        console.log("Task set for Vectis Account on Croncat: ", JSON.stringify(taskRes));
    }
})();
