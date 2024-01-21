import { PluginregistryTypes, PluginRegistryQueryClient as PluginRegC } from "./contracts";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import { pluginRegSubscriptionFee, pluginRegRegistryFee, pluginFreeTierFee } from "../config/fees";
import { fromHex, toHex } from "@cosmjs/encoding";

class PluginRegClient extends PluginRegC {
    constructor(cw: CWClient, _sender: string, contractAddress: string) {
        super(cw.client, contractAddress);
    }

    static createInstMsg(chain: Chain, proxy_code_hash: string, version: string): PluginregistryTypes.InstantiateMsg {
        return {
            registry_fee: pluginRegRegistryFee(chain),
            subscription_tiers: [["free", { fee: pluginFreeTierFee(chain), max_plugins: 3 }]],
            supported_proxies: [[toHex(fromHex(proxy_code_hash)), version]],
        };
    }

    static async instantiate(cw: CWClient, codeId: number, msg: PluginregistryTypes.InstantiateMsg) {
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
