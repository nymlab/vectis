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
import { FactoryT } from "../../interfaces";

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

const update_fee = async (
    client: MsgBroadcasterWithPk,
    sender: string,
    contractAddr: string,
    key: string,
    value: string,
    proposaId: number
) => {
    let message: FactoryT.ExecuteMsg = {
        update_config_fee: {
            new_fee: { denom: "inj", amount: "1" },
            type: "wallet",
        },
    };

    const msg = MsgExecuteContract.fromJSON({
        contractAddress: contractAddr,
        sender,
        msg: {
            propose: {
                description: "update-wallet-fee",
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: contractAddr,
                                funds: [],
                                msg: toCosmosMsg(message),
                            },
                        },
                    },
                ],
                title: "update-fee",
            },
        },
    });

    let result = await client.broadcast({ injectiveAddress: sender, msgs: [msg] });
    console.log(result);

    const executeMsg = MsgExecuteContract.fromJSON({
        contractAddress: contractAddr,
        sender,
        msg: {
            execute: {
                proposal_id: proposaId,
            },
        },
    });

    let exec = await client.broadcast({
        msgs: executeMsg,
        injectiveAddress: sender,
    });
    console.log("execute: ", JSON.stringify(exec));
};

(async function deploy() {
    const network = process.env.HOST_CHAIN;
    console.log("Deploy Vectis to ", network);
    const endpoints = getNetworkEndpoints(Network.TestnetK8s);

    // Admin will be removed at the end of the deploy
    const { admin } = injectiveAccounts[network as keyof typeof injectiveAccounts];
    const adminClient = new MsgBroadcasterWithPk({
        privateKey: PrivateKey.fromMnemonic(admin.mnemonic),
        network: Network.Testnet,
        endpoints,
        simulateTx: true,
    });

    // These are committee members for the vectis and tech committee
    const { committee1 } = injectiveAccounts[network as keyof typeof injectiveAccounts];
    const comittee1Client = new MsgBroadcasterWithPk({
        privateKey: PrivateKey.fromMnemonic(committee1.mnemonic),
        network: Network.Testnet,
        endpoints,
        simulateTx: true,
    });

    //await update_fee(comittee1Client, committee1.address, vectisCommittee, VectisActors.Factory, factoryAddr, 3);
})();
