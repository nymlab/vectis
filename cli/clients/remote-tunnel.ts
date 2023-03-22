import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { RemoteTunnelClient as RemoteTunnelC } from "../interfaces";
import CWClient from "./cosmwasm";
import type { RemoteTunnelT } from "../interfaces";

class RemoteTunnelClient extends RemoteTunnelC {
    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, sender, contractAddress);
    }

    static async instantiate(client: CWClient, codeId: number, msg: RemoteTunnelT.InstantiateMsg) {
        const { contractAddress } = await client.instantiate(
            client.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Vectis Remote tunnel",
            "auto"
        );

        return new RemoteTunnelClient(client, client.sender, contractAddress);
    }
}

export default RemoteTunnelClient;
