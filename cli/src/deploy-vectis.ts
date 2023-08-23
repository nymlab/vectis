import assert from "assert";
import { VectisContractsAddrs, VectisContractsUploadResult } from "./interfaces/vectis-contracts";
import * as ConfigChains from "./config/chains";
import { SupportChains, Chains, Chain } from "./config/chains";
import { Cw4GroupClient, FactoryClient, CWClient, PluginRegClient, Cw3FlexClient } from "./clients";
import { Account } from "./config/accounts";
import { toCosmosMsg } from "./utils/enconding";
import { writeToFile, getAccountsPath, getUploadInfoPath, getDeployPath } from "./utils/fs";
import {
    vectisCommittee1Weight,
    vectisCommittee2Weight,
    vectisTechCommittee1Weight,
    vectisTechCommittee2Weight,
    VectisActors,
} from "./config/vectis";
import { Logger } from "tslog";

export async function deploy(network: Chains) {
    const logger = new Logger();
    if (!(network in SupportChains)) {
        logger.fatal(new Error("Network not supported"));
        throw new Error("Network not supported");
    }

    logger.info("Deploying Vectis Contracts to ", network);

    //  Import uploaded contract results and generated accounts files
    const uploadedContracts: VectisContractsUploadResult = await import(getUploadInfoPath(network, true));
    const hostAccounts: Record<string, Account> = await import(getAccountsPath(network));
    const chain = ConfigChains[network as keyof typeof ConfigChains] as Chain;

    const adminHostClient = await CWClient.connectHostWithAccount("admin", network);
    const committee1Client = await CWClient.connectHostWithAccount("committee1", network);

    // Admin will be removed at the end of the deploy
    const daoAdmin = hostAccounts["admin"] as Account;

    // ===================================================================================
    //
    // Governance committees:  PreProposal + Tech
    //
    // ===================================================================================

    // These are committee members for the vectis and tech committee
    const committee1 = hostAccounts["committee1"] as Account;
    const committee2 = hostAccounts["committee2"] as Account;

    // Instantiate Cw4Group - which will be used for the vectis multisig
    // and technical vectis committee group
    const vectisCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        uploadedContracts.cw4Group.codeId,
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
    logger.info("1. Instantiated group for vectis committees at: ", vectisCommitteeMembers);

    // tech committee group
    const techCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        uploadedContracts.cw4Group.codeId,
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
    logger.info("2. Instantiated group for tech committees at: ", techCommitteeMembers);

    // Instantiate vectis MultiSig
    const vectisCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        uploadedContracts.cw3Flex.codeId,
        vectisCommitteeMembers,
        "Vectis Committee MultiSig"
    );
    const vectisCommittee = vectisCommitteeRes.contractAddress;

    // Instantiate TechCommittee MultiSig
    const techCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        uploadedContracts.cw3Flex.codeId,
        techCommitteeMembers,
        "Tech Committee MultiSig"
    );
    const techCommittee = techCommitteeRes.contractAddress;

    logger.info(
        "3. Instantiated Tech committee Multisig at: ",
        techCommittee,
        "\n Vectis Multisig at: ",
        vectisCommittee
    );

    // Vectis Committee execute deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(
        chain,
        uploadedContracts.vectis_proxy.codeId,
        uploadedContracts.vectis_webauthn_authenticator.codeId
    );

    // ===========================================================
    //
    // Proposals to instantiate Factory and Plugin Registry
    //
    // ===========================================================
    //
    // Vectis Committee deploy factory
    const deployFactoryMsg = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: uploadedContracts.vectis_factory.codeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };

    const vectisComClient = new Cw3FlexClient(committee1Client, committee1.address, vectisCommittee);
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
    const factoryAddr = CWClient.getEventAttrValue(
        execute,
        "wasm-vectis.factory.v1.MsgInstantiate",
        "_contract_address"
    );
    const webauthAddr = CWClient.getEventAttrValue(execute, "wasm-vectis.webauthn.v1", "_contract_address");

    // Vectis Committee deploy plugin registry
    let pluginRegInstMsg = PluginRegClient.createInstMsg(chain);
    const deployPluginRegistry = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: uploadedContracts.vectis_plugin_registry.codeId,
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
    const pluginRegAddr = CWClient.getAddrFromInstantianteResult(plugin);
    logger.info(
        "4. Instantiated factory: ",
        factoryAddr,
        "\nplugin registry: ",
        pluginRegAddr,
        "\nWebauthn: ",
        webauthAddr
    );

    // ===================================================================================
    //
    // Set addresses on vectisCommitteeMultisig
    //      - Single source for Vectis Contract Addresses
    //
    // ===================================================================================
    await vectisComClient.add_item(VectisActors.Factory, factoryAddr);
    await vectisComClient.add_item(VectisActors.PluginRegistry, pluginRegAddr);
    await vectisComClient.add_item(VectisActors.PluginCommittee, techCommittee);
    logger.info("\n5. Set dao_tunnel and committee address in DAO items");

    const contracts: VectisContractsAddrs = {
        PluginCommittee: techCommittee,
        PluginCommitteeGroup: techCommitteeMembers,
        VectisCommittee: vectisCommittee,
        VectisCommitteeGroup: vectisCommitteeMembers,
        PluginRegistry: pluginRegAddr,
        Factory: factoryAddr,
        Webauthn: webauthAddr,
    };
    logger.info("\n 6. Contracts on Chain: ", contracts);
    writeToFile(getDeployPath(network), JSON.stringify(contracts, null, 2));

    await verify(contracts, vectisComClient, network);
    logger.info("\n 7. END. Verified deployment");
}

async function verify(addrs: VectisContractsAddrs, vectisClient: Cw3FlexClient, network: Chains) {
    const adminHostClient = await CWClient.connectHostWithAccount("admin", network);

    // Checks contract admins for migrations
    for (const [key, value] of Object.entries(addrs)) {
        // Ensure VectisCommittee is admin so that it can update contracts.
        if (key == "Factory" || key == "PluginRegistry") {
            let contract = await adminHostClient.client.getContract(value);
            assert.strictEqual(contract.admin, addrs.VectisCommittee);
        }
    }

    // Committe multisig should have all VectisActors addresses
    for (const [key, value] of Object.entries(VectisActors)) {
        let addr = await vectisClient.getItem({ key: value });
        assert.strictEqual(addrs[key as keyof typeof addrs], addr);
    }
}
