import assert from "assert";
import { FactoryClient, CWClient, Cw3FlexClient, Cw4GroupClient, PluginRegClient } from "./clients";
import { Account } from "./config/accounts";
import { toCosmosMsg } from "./utils/enconding";
import { writeToFile } from "./utils/fs";
import { hubDeployReportPath, hubUploadReportPath } from "./utils/constants";
import * as chainConfigs from "./config/chains";
import * as accountConfigs from "./config/accounts";
import type { Chain } from "./config/chains";

import type { VectisHubChainContractsAddrs } from "./interfaces/contracts";
import {
    vectisCommittee1Weight,
    vectisCommittee2Weight,
    vectisTechCommittee1Weight,
    vectisTechCommittee2Weight,
} from "./clients/cw3flex";
import { VectisActors } from "./utils/constants";

export async function deploy(network: string) {
    const chain = chainConfigs[network as keyof typeof chainConfigs] as Chain;
    const { factory, proxy, cw3Fixed, pluginReg, cw4Group, cw3Flex } = await import(hubUploadReportPath(chain));

    const adminHostClient = await CWClient.connectHostWithAccount("admin", network);
    const committee1Client = await CWClient.connectHostWithAccount("committee1", network);
    let balance = await committee1Client.client.getBalance(committee1Client.sender, chain.feeToken);
    console.log("balance: ", balance);

    const expectedAmount = Math.ceil(2 * Math.pow(10, chain.exponent));
    if (+balance.amount < expectedAmount) {
        await adminHostClient.client.sendTokens(
            adminHostClient.sender,
            committee1Client.sender,
            [
                {
                    denom: chain.feeToken,
                    amount: expectedAmount.toString(),
                },
            ],
            "auto"
        );
    }
    balance = await committee1Client.client.getBalance(committee1Client.sender, chain.feeToken);
    console.log("balance: ", balance);

    // Admin will be removed at the end of the deploy
    const hostAccounts = accountConfigs[network as keyof typeof accountConfigs];
    const daoAdmin = hostAccounts["admin"];

    // ===================================================================================
    //
    // Governance committees:  PreProposal + Tech
    //
    // ===================================================================================

    // These are committee members for the vectis and tech committee
    const committee1 = hostAccounts["committee1"] as Account;
    const committee2 = hostAccounts["committee2"] as Account;

    // Instantiate Cw4Group - which will be used for the vectis multisig and technical

    // vectis committee group
    const vectisCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        cw4Group.codeId,
        daoAdmin.address,
        [
            {
                addr: committee1.address,
                weight: vectisCommittee1Weight,
            },
            {
                addr: committee2.address,
                weight: vectisCommittee2Weight,
            },
        ],
        "Vectis Committee"
    );
    const vectisCommitteeMembers = vectisCommitteeMemberRes.contractAddress;
    console.log("1. Instantiated group for vectis committees at: ", vectisCommitteeMembers);

    // tech committee group
    const techCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        cw4Group.codeId,
        daoAdmin.address,
        [
            {
                addr: committee1.address,
                weight: vectisTechCommittee1Weight,
            },
            {
                addr: committee2.address,
                weight: vectisTechCommittee2Weight,
            },
        ],
        "Vectis Tech Committee"
    );
    const techCommitteeMembers = techCommitteeMemberRes.contractAddress;
    console.log("2. Instantiated group for tech committees at: ", techCommitteeMembers);
    console.log("cw4", JSON.stringify(techCommitteeMemberRes));

    // Instantiate vectis MultiSig
    const vectisCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        cw3Flex.codeId,
        vectisCommitteeMembers,
        "Vectis Committee MultiSig"
    );
    const vectisCommittee = vectisCommitteeRes.contractAddress;
    const vectisComClient = new Cw3FlexClient(committee1Client, committee1.address, vectisCommittee);

    // Instantiate TechCommittee MultiSig
    const techCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        cw3Flex.codeId,
        techCommitteeMembers,
        "Tech Committee MultiSig"
    );
    const techCommittee = techCommitteeRes.contractAddress;
    console.log("cw3", JSON.stringify(techCommitteeRes));

    console.log(
        "3. Instantiated Tech committee Multisig at: ",
        techCommittee,
        "\n Vectis Multisig at: ",
        vectisCommittee
    );

    // Vectis Committee execute deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(chain, proxy.codeId, cw3Fixed.codeId);
    const deployFactoryMsg = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: factory.codeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };
    await vectisComClient.propose(
        {
            description: "deploy factory",
            latest: undefined,
            msgs: [deployFactoryMsg],
            title: "deploy hub factory",
        },
        "auto"
    );
    const proposals = await vectisComClient.listProposals({});
    const prop = proposals.proposals.pop();
    const propId = prop!.id;
    let execute = await vectisComClient.execute({ proposalId: propId });
    console.log("execute proposal: ", JSON.stringify(execute));
    const factoryAddr = CWClient.getContractAddrFromResult(execute, "_contract_address");

    // Vectis Committee deploy plugin registry
    let pluginRegInstMsg = PluginRegClient.createInstMsg(chain);
    const deployPluginRegistry = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: pluginReg.codeId,
                funds: [],
                label: "Vectis Plugin Registry",
                msg: toCosmosMsg(pluginRegInstMsg),
            },
        },
    };
    await vectisComClient.propose(
        {
            description: "deploy plugin_reg",
            latest: undefined,
            msgs: [deployPluginRegistry],
            title: "deploy hub plugin reg",
        },
        "auto"
    );
    const pluginProps = await vectisComClient.listProposals({});
    const pluginProp = pluginProps.proposals.pop();
    let plugin = await vectisComClient.execute({ proposalId: pluginProp!.id });
    const pluginRegAddr = CWClient.getContractAddrFromResult(plugin, "_contract_address");
    console.log("4. Instantiated factory: ", factoryAddr, "\n plugin registry: ", pluginRegAddr);

    // ===================================================================================
    //
    // Set addresses on vectisCommitteeMultisig
    //
    // ===================================================================================
    await vectisComClient.add_item(VectisActors.Factory, factoryAddr);
    await vectisComClient.add_item(VectisActors.PluginRegistry, pluginRegAddr);
    await vectisComClient.add_item(VectisActors.PluginCommittee, techCommittee);
    console.log("\n5. Set Factory, pluginReg and committee address in DAO items");

    const contracts = {
        // These are the same as DaoActors
        PluginCommittee: techCommittee,
        PluginCommitteeGroup: techCommitteeMembers,
        VectisCommittee: vectisCommittee,
        VectisCommitteeGroup: vectisCommitteeMembers,
        PluginRegistry: pluginRegAddr,
        Factory: factoryAddr,
    };
    console.log("\n Contracts on Chain: ", contracts);
    writeToFile(hubDeployReportPath(chain), JSON.stringify(contracts, null, 2));

    await verify(contracts, vectisComClient, network);
    console.log("\nEND. Verified deployment");
}

async function verify(addrs: VectisHubChainContractsAddrs, vectisClient: Cw3FlexClient, network: string) {
    console.log("Verifying deployment");
    const adminHostClient = await CWClient.connectHostWithAccount("admin", network);

    // Checks contract admins for migrations
    for (const [key, value] of Object.entries(addrs)) {
        // Ensure VectisCommittee is admin so that it can update contracts.
        if (key == "Factory" || key == "PluginRegistry") {
            let contract = await adminHostClient.client.getContract(value);
            console.log("contract: ", key, "; Admin: ", contract.admin);
            assert.strictEqual(contract.admin, addrs.VectisCommittee);
        } else {
            let contract = await adminHostClient.client.getContract(value);
            console.log("contract: ", key, "; Admin: ", contract.admin);
        }
    }
    console.log("Factory and PluginRegistry is upgradable by committees");

    // DAO should have all DaoActors addresses
    for (const [key, value] of Object.entries(VectisActors)) {
        let addr = await vectisClient.getItem({ key: value });
        assert.strictEqual(addrs[key as keyof typeof addrs], addr);
    }
    console.log("VectisActors stored correctly on VectisCommittee");
}
