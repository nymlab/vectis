import { CWClient, Cw3FlexClient, PluginRegClient } from "../clients";
import { pluginRegRegistryFee } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import { hubDeployReportPath, hostChain } from "../utils/constants";
import { ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../interfaces/Cw3Flex.types";
import { ExecuteMsg as PluginRegistryExecMsg } from "../interfaces/PluginRegistry.types";

const checksum = "2a205bc31f9e94adc62ab6db42c518d9787d41a8149fdb87e7eceb035d6f3a5b";
const code_id = 13;
const creator = "juno1dfd5vtxy2ty5gqqv0cs2z23pfucnpym9kcq8vv";
const ipfs_hash = "n/a";
const name = "Cronkitty";
const version = "0.1";

(async function add_plugin() {
    const { PluginCommitteeGroup, PluginCommittee, PluginRegistry } = await import(hubDeployReportPath);

    const tc1Client = await CWClient.connectHostWithAccount("committee1");
    const tc2Client = await CWClient.connectHostWithAccount("committee2");
    const cw3client = new Cw3FlexClient(tc1Client, tc1Client.sender, PluginCommittee);
    const prClient = new PluginRegClient(tc1Client, tc1Client.sender, PluginRegistry);

    // Make sure there is balance
    // TODO do checks on balances
    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const funds = { amount: (1800000).toString(), denom: hostChain.feeToken };
    await adminHostClient.sendTokens(adminHostClient.sender, tc1Client.sender, [funds], "auto");
    await adminHostClient.sendTokens(adminHostClient.sender, tc2Client.sender, [funds], "auto");

    const proposals = await cw3client.reverseProposals({ startBefore: undefined, limit: undefined });
    console.log("proposals, ", JSON.stringify(proposals));

    let currentId = proposals.proposals.length;

    let pluginRegExecMsg: PluginRegistryExecMsg = {
        register_plugin: {
            checksum,
            code_id,
            creator,
            ipfs_hash,
            name,
            version,
        },
    };

    let registerFund = pluginRegRegistryFee(hostChain).amount == "0" ? [] : [pluginRegRegistryFee(hostChain)];
    let execMsgForPluginReg: CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: PluginRegistry,
                funds: [],
                msg: toCosmosMsg(pluginRegExecMsg),
            },
        },
    };

    let proposalMsg: Cw3FlexExecMsg = {
        propose: {
            description: "Add CronKitty Tx automation plugin",
            msgs: [execMsgForPluginReg],
            title: "Add Plugin",
        },
    };

    let execMsg: Cw3FlexExecMsg = {
        execute: {
            proposal_id: currentId + 1,
        },
    };

    console.log("proposal id: ", currentId + 1);

    let fees = pluginRegRegistryFee(hostChain).amount == "0" ? undefined : [pluginRegRegistryFee(hostChain)];
    await tc1Client.execute(tc1Client.sender, PluginCommittee, proposalMsg, "auto", undefined, undefined);

    const proposalsAfter = await cw3client.reverseProposals({ startBefore: undefined, limit: undefined });
    console.log("proposals After, ", JSON.stringify(proposalsAfter));

    await tc1Client.execute(tc1Client.sender, PluginCommittee, execMsg, "auto");

    let plugins = await prClient.getPlugins({});
    console.log("Plugins: \n", JSON.stringify(plugins));
    writeInCacheFolder(`registeredPlugins.json`, JSON.stringify(plugins, null, 2));
})();
