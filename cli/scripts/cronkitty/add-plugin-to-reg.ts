import { CWClient, Cw3FlexClient, PluginRegClient } from "../../clients";
import { pluginRegRegistryFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { writeInCacheFolder } from "../../utils/fs";
import { Account } from "../../config/accounts";
import { hostAccounts, hubUploadReportPath, hubDeployReportPath, hostChain } from "../../utils/constants";
import { ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { ExecuteMsg as PluginRegistryExecMsg } from "../../interfaces/PluginRegistry.types";

// v0.2.1
const ipfs_hash = "QmYiz6JcREYvdoyonqfgmmoa4HkvfKkyMDGRf2a8RzFnmK";
const name = "cronkitty";
const version = "0.2.1";

(async function add_plugin() {
    const { PluginCommittee, PluginRegistry } = await import(hubDeployReportPath);
    const uploads = await import(hubUploadReportPath);
    let uploadRes = uploads.plugins["cronkitty"];
    console.log("uploadRes", uploadRes);

    if (uploadRes) {
        const code_id = uploadRes.codeId;
        const checksum = uploadRes.originalChecksum;
        const creator = hostAccounts["admin"] as Account;
        const tc1Client = await CWClient.connectHostWithAccount("committee1");
        const cw3client = new Cw3FlexClient(tc1Client, tc1Client.sender, PluginCommittee);
        const prClient = new PluginRegClient(tc1Client, tc1Client.sender, PluginRegistry);

        const proposals = await cw3client.reverseProposals({ startBefore: undefined, limit: undefined });
        let currentId = proposals.proposals.length == 0 ? 0 : proposals.proposals[0].id;

        let pluginRegExecMsg: PluginRegistryExecMsg = {
            //unregister_plugin: { id: 1 },
            register_plugin: {
                checksum,
                code_id,
                creator: creator.address,
                ipfs_hash,
                name,
                version,
            },
        };

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
                description: "Add Plugin CronKitty Tx automation plugin",
                msgs: [execMsgForPluginReg],
                title: "Add CronKitty Plugin",
            },
        };

        let execMsg: Cw3FlexExecMsg = {
            execute: {
                proposal_id: currentId + 1,
            },
        };

        console.log("proposal id: ", currentId);

        let fees = pluginRegRegistryFee(hostChain).amount == "0" ? undefined : [pluginRegRegistryFee(hostChain)];
        await tc1Client.execute(tc1Client.sender, PluginCommittee, proposalMsg, "auto", undefined, undefined);

        const proposalsAfter = await cw3client.reverseProposals({ startBefore: undefined, limit: undefined });
        console.log("proposals After, ", JSON.stringify(proposalsAfter));

        await tc1Client.execute(tc1Client.sender, PluginCommittee, execMsg, "auto");

        let plugins = await prClient.getPlugins({});
        console.log("Plugins: \n", JSON.stringify(plugins));
        //writeInCacheFolder(`registeredPlugins.json`, JSON.stringify(plugins, null, 2));
    }
})();
