import {
    vectisCommittee1Weight,
    vectisCommittee2Weight,
    vectisTechCommittee1Weight,
    vectisTechCommittee2Weight,
} from "../../clients/cw3flex";

import * as injectiveAccounts from "../../config/accounts/injective";
import { MsgBroadcasterWithPk, MsgExecuteContract, MsgInstantiateContract, PrivateKey } from "@injectivelabs/sdk-ts";
import { Network, getNetworkEndpoints } from "@injectivelabs/networks";
import { toCosmosMsg } from "../../utils/enconding";
import { writeToFile } from "../../utils/fs";
import { VectisActors } from "../../utils/constants";

interface CodeIds {
    proxyCodeId: number;
    pluginRegCodeId: number;
    factoryCodeId: number;
    cw3FixedCodeId: number;
    cw3FlexCodeId: number;
    cw4GroupCodeId: number;
}

const extractValueFromEvent = (rawLog: string, event: string, attribute: string) => {
    const events = JSON.parse(rawLog)[0].events as { type: string; attributes: { key: string; value: string }[] }[];
    const e = events.find((e) => e.type === event);
    if (!e) throw new Error("It was not possible to find the event");
    const a = e.attributes.find((attr) => attr.key === attribute);
    if (!a) throw new Error("It was not possible to find the attribute");
    return JSON.parse(a.value);
};

const addItem = async (
    client: MsgBroadcasterWithPk,
    sender: string,
    contractAddr: string,
    key: string,
    value: string,
    proposaId: number
) => {
    const msg = MsgExecuteContract.fromJSON({
        contractAddress: contractAddr,
        sender,
        msg: {
            propose: {
                description: "add-item",
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: contractAddr,
                                funds: [],
                                msg: toCosmosMsg({ update_item: { key, value } }),
                            },
                        },
                    },
                ],
                title: "add-item",
            },
        },
    });

    await client.broadcast({ injectiveAddress: sender, msgs: [msg] });

    const executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: contractAddr,
        sender,
        msg: {
            execute: {
                proposal_id: proposaId,
            },
        },
    });

    await client.broadcast({
        msgs: executeMsg,
        injectiveAddress: sender,
    });
};

(async function deploy() {
    const network = process.env.HOST_CHAIN;
    console.log("Deploy Vectis to ", network);
    const endpoints = getNetworkEndpoints(Network.TestnetK8s);

    const { factoryCodeId, proxyCodeId, cw3FixedCodeId, pluginRegCodeId, cw4GroupCodeId, cw3FlexCodeId } =
        (await import("./../../deploy/injective_testnet-uploadInfo.json")) as CodeIds;

    // Admin will be removed at the end of the deploy
    const { admin } = injectiveAccounts[network as keyof typeof injectiveAccounts];
    const adminClient = new MsgBroadcasterWithPk({
        privateKey: PrivateKey.fromMnemonic(admin.mnemonic),
        network: Network.Testnet,
        endpoints,
        simulateTx: true,
    });

    // ===================================================================================
    //
    // Governance committees:  PreProposal + Tech
    //
    // ===================================================================================

    // These are committee members for the vectis and tech committee
    const { committee1, committee2 } = injectiveAccounts[network as keyof typeof injectiveAccounts];
    const comittee1Client = new MsgBroadcasterWithPk({
        privateKey: PrivateKey.fromMnemonic(committee1.mnemonic),
        network: Network.Testnet,
        endpoints,
        simulateTx: true,
    });

    // Instantiate Cw4Group - which will be used for the vectis multisig and technical

    // vectis committee group
    let msg = MsgInstantiateContract.fromJSON({
        sender: admin.address,
        codeId: cw4GroupCodeId,
        label: "Vectis Committee",
        admin: admin.address,
        msg: {
            admin: admin.address,
            members: [
                {
                    addr: committee1.address,
                    weight: vectisCommittee1Weight,
                },
                {
                    addr: committee2.address,
                    weight: vectisCommittee2Weight,
                },
            ],
        },
    });

    let txResponse = await adminClient.broadcast({
        msgs: msg,
        injectiveAddress: admin.address,
    });

    const vectisCommitteeMembers = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );
    console.log("1. Instantiated group for vectis committees at: ", vectisCommitteeMembers);

    // tech committee group
    msg = MsgInstantiateContract.fromJSON({
        sender: admin.address,
        codeId: cw4GroupCodeId,
        label: "Vectis Tech Committee",
        admin: admin.address,
        msg: {
            admin: admin.address,
            members: [
                {
                    addr: committee1.address,
                    weight: vectisTechCommittee1Weight,
                },
                {
                    addr: committee2.address,
                    weight: vectisTechCommittee2Weight,
                },
            ],
        },
    });

    txResponse = await adminClient.broadcast({
        msgs: msg,
        injectiveAddress: admin.address,
    });

    const techCommitteeMembers = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );
    console.log("2. Instantiated group for tech committees at: ", techCommitteeMembers);

    // Instantiate vectis MultiSig

    // Proposal for the Hub multisig config
    // Length of  max Voting Period, Time in seconds
    const maxVotingPeriod = {
        time: 60 * 60 * 24 * 14,
    };

    // Vectis Committee Config
    // Responsible for approving plugins into the Plugin registry
    const vectisCommitteeThreshold = {
        absolute_percentage: { percentage: "0.5" },
    };

    msg = MsgInstantiateContract.fromJSON({
        sender: admin.address,
        codeId: cw3FlexCodeId,
        label: "Pre Proposal MultiSig",
        admin: admin.address,
        msg: {
            executor: null,
            group_addr: vectisCommitteeMembers,
            max_voting_period: maxVotingPeriod,
            threshold: vectisCommitteeThreshold,
        },
    });

    txResponse = await adminClient.broadcast({
        msgs: msg,
        injectiveAddress: admin.address,
    });

    const vectisCommittee = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );

    // Instantiate TechCommittee MultiSig

    msg = MsgInstantiateContract.fromJSON({
        sender: admin.address,
        codeId: cw3FlexCodeId,
        label: "Tech Committee MultiSig",
        admin: admin.address,
        msg: {
            executor: null,
            group_addr: techCommitteeMembers,
            max_voting_period: maxVotingPeriod,
            threshold: vectisCommitteeThreshold,
        },
    });

    txResponse = await adminClient.broadcast({
        msgs: msg,
        injectiveAddress: admin.address,
    });

    const techCommittee = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );

    console.log(
        "3. Instantiated Tech committee Multisig at: ",
        techCommittee,
        "\n Vectis Multisig at: ",
        vectisCommittee
    );

    // Vectis Committee execute deploy factory
    const factoryInstMsg = {
        proxy_code_id: proxyCodeId,
        proxy_multisig_code_id: cw3FixedCodeId,
        addr_prefix: "inj",
        wallet_fee: { amount: "10", denom: "inj" },
    };
    const deployFactoryMsg = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: factoryCodeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };

    const proposeMsg = {
        propose: {
            description: "Deploy Factory",
            msgs: [deployFactoryMsg],
            title: "Deploy Hub Factory",
        },
    };

    let executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: vectisCommittee,
        sender: committee1.address,
        msg: proposeMsg,
    });

    txResponse = await comittee1Client.broadcast({
        msgs: executeMsg,
        injectiveAddress: committee1.address,
    });

    executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: vectisCommittee,
        sender: committee1.address,
        msg: {
            execute: {
                proposal_id: 1,
            },
        },
    });

    txResponse = await comittee1Client.broadcast({
        msgs: executeMsg,
        injectiveAddress: committee1.address,
    });

    const factoryAddr = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );

    // Vectis Committee deploy plugin registry
    const pluginRegInstMsg = {
        install_fee: { amount: "10", denom: "inj" },
        registry_fee: { amount: "0", denom: "inj" },
    };

    const deployPluginRegistry = {
        wasm: {
            instantiate: {
                admin: vectisCommittee,
                code_id: pluginRegCodeId,
                funds: [],
                label: "Vectis Plugin Registry",
                msg: toCosmosMsg(pluginRegInstMsg),
            },
        },
    };

    executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: vectisCommittee,
        sender: committee1.address,
        msg: {
            propose: {
                description: "deploy plugin_reg",
                msgs: [deployPluginRegistry],
                title: "deploy hub plugin reg",
            },
        },
    });

    txResponse = await comittee1Client.broadcast({
        msgs: executeMsg,
        injectiveAddress: committee1.address,
    });

    executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: vectisCommittee,
        sender: committee1.address,
        msg: {
            execute: {
                proposal_id: 2,
            },
        },
    });

    txResponse = await comittee1Client.broadcast({
        msgs: executeMsg,
        injectiveAddress: committee1.address,
    });

    const pluginRegAddr = extractValueFromEvent(
        txResponse.rawLog,
        "cosmwasm.wasm.v1.EventContractInstantiated",
        "contract_address"
    );

    console.log("4. Instantiated factory: ", factoryAddr, "\n plugin registry: ", pluginRegAddr);

    // ===================================================================================
    //
    // Set addresses on vectisCommitteeMultisig
    //
    // ===================================================================================

    await addItem(comittee1Client, committee1.address, vectisCommittee, VectisActors.Factory, factoryAddr, 3);
    await addItem(comittee1Client, committee1.address, vectisCommittee, VectisActors.PluginRegistry, pluginRegAddr, 4);
    await addItem(comittee1Client, committee1.address, vectisCommittee, VectisActors.PluginCommittee, techCommittee, 5);

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
    writeToFile(`./deploy/${network}-deployInfo.json`, JSON.stringify(contracts, null, 2));

    // ===================================================================================
    //
    // Add identity plugin to the registry = code-id = 998 on injective-testnet
    //
    // ===================================================================================

    // Identity Plugin
    const checksum = "d5fe0daac6794324fef16580e8a595e1a2f70572696e4d5d92b9b7645bbf4286";
    const code_id = 1139;
    const creator = "inj1dr5u7sxpmmckrvj0cc9he6sdl8qnje9wlplajv";
    const ipfs_hash = "test-ipfs-hash";
    const name = "Avida Identity Plugin";
    const version = "0.1.0";
    const regMsg = MsgExecuteContract.fromJSON({
        contractAddress: techCommittee,
        sender: committee1.address,
        msg: {
            propose: {
                description: "Add AVIDA plugin",
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: pluginRegAddr,
                                funds: [],
                                msg: toCosmosMsg({
                                    register_plugin: {
                                        checksum,
                                        code_id,
                                        creator,
                                        ipfs_hash,
                                        name,
                                        version,
                                    },
                                }),
                            },
                        },
                    },
                ],
                title: "Add avida identity plugin",
            },
        },
    });

    let proposal_tx = await comittee1Client.broadcast({ injectiveAddress: committee1.address, msgs: [regMsg] });
    const regExecuteMsg = MsgExecuteContract.fromJSON({
        contractAddress: techCommittee,
        sender: committee1.address,
        msg: {
            execute: {
                proposal_id: 1,
            },
        },
    });

    let execute_tx = await comittee1Client.broadcast({
        msgs: regExecuteMsg,
        injectiveAddress: committee1.address,
    });

    console.log(" Proposal tx: ", JSON.stringify(execute_tx));
    writeToFile(`../../deploy/${network}-IdentityPlugin.json`, JSON.stringify(execute_tx, null, 2));
})();
