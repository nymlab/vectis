import CWClient from "./cosmwasm";
import { Cw4GroupClient as Cw4GroupC, Cw4GroupT } from "../interfaces";

class Cw4GroupClient extends Cw4GroupC {
    constructor(cw: CWClient, sender: string, contractAddr: string) {
        super(cw.client, sender, contractAddr);
    }

    static async instantiate(
        cw: CWClient,
        codeId: number,
        admin: string | null,
        members: Cw4GroupT.Member[],
        label: string
    ) {
        const instantiate: Cw4GroupT.InstantiateMsg = {
            admin: admin,
            members: members,
        };

        const { contractAddress } = await cw.client.instantiate(
            cw.sender,
            codeId,
            instantiate as unknown as Record<string, unknown>,
            label,
            "auto",
            {
                admin: cw.sender,
            }
        );

        return new Cw4GroupClient(cw, cw.sender, contractAddress);
    }
}

export default Cw4GroupClient;
