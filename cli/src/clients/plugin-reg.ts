import { PluginRegT, PluginRegistryQueryClient as PluginRegC } from "../interfaces";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import { pluginRegSubscriptionFee, pluginRegRegistryFee } from "../config/fees";

class PluginRegClient extends PluginRegC {
    constructor(cw: CWClient, _sender: string, contractAddress: string) {
        super(cw.client, contractAddress);
    }

    static createInstMsg(chain: Chain): PluginRegT.InstantiateMsg {
        return {
            subscription_fee: pluginRegSubscriptionFee(chain),
            registry_fee: pluginRegRegistryFee(chain),
        };
    }

    static async instantiate(cw: CWClient, codeId: number, msg: PluginRegT.InstantiateMsg) {
        const { contractAddress } = await cw.client.instantiate(
            cw.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Vectis Plugin Registry",
            "auto",
            {
                funds: [],
            }
        );

        return new PluginRegClient(cw, cw.sender, contractAddress);
    }
}

export default PluginRegClient;
