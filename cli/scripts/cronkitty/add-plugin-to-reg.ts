import { CWClient, Cw3FlexClient, PluginRegClient } from "../../clients";
import { pluginRegRegistryFee } from "../../utils/fees";
import { toCosmosMsg } from "../../utils/enconding";
import { writeInCacheFolder } from "../../utils/fs";
import { Account } from "../../config/accounts";
import { hostAccounts, hubUploadReportPath, hubDeployReportPath, hostChain } from "../../utils/constants";
import { ExecuteMsg as Cw3FlexExecMsg, CosmosMsgForEmpty } from "../../interfaces/Cw3Flex.types";
import { ExecuteMsg as PluginRegistryExecMsg } from "../../interfaces/PluginRegistry.types";

// v0.2.1
const checksum = "7d0112936d0966f2a6c7d7b34f9133e71b6324a5e11d0fcb3040d63d479a910d";
const ipfs_hash = "test-hash";
const name = "cronkitty";
const version = "0.2.1";

(async function add_plugin() {
    const { PluginCommittee, PluginRegistry } = await import(hubDeployReportPath);
    const uploads = await import(hubUploadReportPath);
    let uploadRes = uploads.plugins["cronkitty"];

    if (uploadRes) {
        const code_id = uploadRes.codeId;
        const creator = hostAccounts["admin"] as Account;
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
        let currentId = proposals.proposals.length;

        let pluginRegExecMsg: PluginRegistryExecMsg = {
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
                description: "Add CronKitty Tx automation plugin",
                msgs: [execMsgForPluginReg],
                title: "Add CronKitty Plugin",
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
    }
})();
