import { CWClient, Cw3FlexClient, PluginRegClient } from "../clients";
import { pluginRegRegistryFee } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import { daoDeployReportPath, hostChain } from "../utils/constants";
import { ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../interfaces/Cw3Flex.types";
import { ExecuteMsg as PluginRegistryExecMsg } from "../interfaces/PluginRegistry.types";

const checksum = "cb1f0920407d013b2a77dc770d197eaf676fba9c66a5162312215985323b8a0c";
const code_id = 1126;
const creator = "juno1dfd5vtxy2ty5gqqv0cs2z23pfucnpym9kcq8vv";
const ipfs_hash = "n/a";
const name = "Cronkitty";
const version = "0.1";

(async function add_plugin() {
    const { PluginCommitteeGroup, PluginCommittee, PluginRegistry } = await import(daoDeployReportPath);

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

    let execMsgForPluginReg: CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: PluginRegistry,
                funds: [pluginRegRegistryFee(hostChain)],
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

    let voteMsg: Cw3FlexExecMsg = {
        vote: {
            proposal_id: currentId + 1,
            vote: "yes",
        },
    };

    let execMsg: Cw3FlexExecMsg = {
        execute: {
            proposal_id: currentId + 1,
        },
    };

    await tc1Client.execute(tc1Client.sender, PluginCommittee, proposalMsg, "auto", undefined, [
        pluginRegRegistryFee(hostChain),
    ]);

    const proposalsAfter = await cw3client.reverseProposals({ startBefore: undefined, limit: undefined });
    console.log("proposals After, ", JSON.stringify(proposalsAfter));

    await tc2Client.execute(tc2Client.sender, PluginCommittee, voteMsg, "auto");
    await tc2Client.execute(tc2Client.sender, PluginCommittee, execMsg, "auto");

    let plugins = await prClient.getPlugins({});
    console.log("Plugins: \n", JSON.stringify(plugins));
    writeInCacheFolder(`registeredPlugins.json`, JSON.stringify(plugins, null, 2));
})();
