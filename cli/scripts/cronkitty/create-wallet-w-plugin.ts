import { CWClient, FactoryClient, ProxyClient } from "../../clients";
import { pluginRegInstallFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { hubDeployReportPath, hostChain, hostChainName } from "../../utils/constants";
import { ExecuteMsg as CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { ProxyT, CroncatT } from "../../interfaces";
import { createSingleProxyWallet } from "../../tests/mocks/proxyWallet";

const croncat_factory_addr = "neutron1sc3r0m8zxw34jfg5xtym8tuxg38n2efuazap8nzmcgrjfampc0vqp0lg55";
const plugin_id = 2;

(async function create_wallet_with_plugin() {
    const { Factory } = await import(hubDeployReportPath);

    const userClient = await CWClient.connectHostWithAccount("user");

    //// Create Vectis Account
    const factoryClient = new FactoryClient(userClient, userClient.sender, Factory);
    let walletAddr = await createSingleProxyWallet(factoryClient, "host", "neutron-606-cronkitty");

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
    let plugins = await proxyClient.plugins();
    console.log("All plugins: ", plugins);
    console.log("Cronkitty Plugin: ", cronkittyAddr);
    console.log("Vectis Account: ", walletAddr);

    if (hostChainName != "juno_localnet") {
        // Create Task on Cronkitty
        // This sends 0.001 juno to wallet itself every
        const funds = { amount: (1000).toString(), denom: hostChain.feeToken };
        const gas = { amount: (1000000).toString(), denom: hostChain.feeToken };
        let task: CroncatT.TaskRequest = {
            actions: [
                {
                    gas_limit: 1000000,
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
            boundary: null,
            interval: { block: 10 },
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
        const action = await userClient.queryContractSmart(cronkittyAddr!, { action: { action_id: actionId - 1 } });
        console.log("Task set for Vectis Account on Cronkitty: ", JSON.stringify(action));

        const task_addr = action.task_addr;
        const taskRes = await userClient.queryContractSmart(task_addr, {
            task: { task_hash: action.task_hash },
        });
        console.log("Task set for Vectis Account on Croncat: ", JSON.stringify(taskRes));
    }
})();
