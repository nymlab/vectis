import CWClient from "./cosmwasm";
import { Cw3FlexClient as Cw3FlexC, Cw3FlexT } from "../interfaces";
import { preProMaxVotingPeriod, prePropThreshold } from "../clients/dao";

class Cw3FlexClient extends Cw3FlexC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    static async instantiate(client: CWClient, codeId: number, cw4Addr: string, label: string) {
        const instantiate: Cw3FlexT.InstantiateMsg = {
            executor: null,
            group_addr: cw4Addr,
            max_voting_period: preProMaxVotingPeriod,
            threshold: prePropThreshold,
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

        return new Cw3FlexClient(client, client.sender, contractAddress);
    }
}

export default Cw3FlexClient;
