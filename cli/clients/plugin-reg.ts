import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { PluginRegT, PluginRegistryClient as PluginRegC } from "../interfaces";
import { coin } from "@cosmjs/stargate";

import CWClient from "./cosmwasm";
import * as CHAINS from "../config/chains";

import type { Chains } from "../config/chains";
import { pluginRegInstallFee, pluginRegRegistryFee } from "../utils/fees";

class PluginRegClient extends PluginRegC {
    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, sender, contractAddress);
    }

    static createInstMsg(chainName: Chains): PluginRegT.InstantiateMsg {
        const { addressPrefix, feeToken } = CHAINS[chainName];
        return {
            install_fee: pluginRegInstallFee(CHAINS[chainName]),
            registry_fee: pluginRegRegistryFee(CHAINS[chainName]),
        };
    }

    static async instantiate(client: CWClient, codeId: number, msg: PluginRegT.InstantiateMsg) {
        const { contractAddress } = await client.instantiate(
            client.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Wallet Factory",
            "auto",
            {
                funds: [],
            }
        );

        return new PluginRegClient(client, client.sender, contractAddress);
    }
}

export default PluginRegClient;
