import CWClient from "./cosmwasm";
import { Cw4GroupClient as Cw4GroupC, Cw4GroupT } from "../interfaces";

class Cw4GroupClient extends Cw4GroupC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    static async instantiate(
        client: CWClient,
        codeId: number,
        cw4Addr: string,
        admin: string | null,
        members: Cw4GroupT.Member[],
        label: string
    ) {
        const instantiate: Cw4GroupT.InstantiateMsg = {
            admin: admin,
            members: members,
        };

        const { contractAddress } = await client.instantiate(
            client.sender,
            codeId,
            instantiate as unknown as Record<string, unknown>,
            label,
            "auto",
            {
                admin: client.sender,
            }
        );

        return new Cw4GroupClient(client, client.sender, contractAddress);
    }
}

export default Cw4GroupClient;
