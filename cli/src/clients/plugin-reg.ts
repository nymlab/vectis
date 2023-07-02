import { PluginRegT, PluginRegistryClient as PluginRegC } from "../interfaces";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import { pluginRegInstallFee, pluginRegRegistryFee } from "../utils/fees";

class PluginRegClient extends PluginRegC {
    constructor(cw: CWClient, sender: string, contractAddress: string) {
        super(cw.client, sender, contractAddress);
    }

    static createInstMsg(chain: Chain): PluginRegT.InstantiateMsg {
        return {
            install_fee: pluginRegInstallFee(chain),
            registry_fee: pluginRegRegistryFee(chain),
        };
    }

    static async instantiate(cw: CWClient, codeId: number, msg: PluginRegT.InstantiateMsg) {
        const { contractAddress } = await cw.client.instantiate(
            cw.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Wallet Factory",
            "auto",
            {
                funds: [],
            }
        );

        return new PluginRegClient(cw, cw.sender, contractAddress);
    }
}

export default PluginRegClient;
