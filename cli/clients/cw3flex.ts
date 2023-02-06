import CWClient from "./cosmwasm";
import { Cw4GroupClient as Cw4GroupC, Cw4GroupT } from "../interfaces";

class Cw3FlexClient extends Cw3FlexC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    static async instantiate(
        client: CWClient,
        codeId: number,
        cw4Addr: string,
        maxVotingPeriod: Cw3FlexT.Duration,
        threshold: Cw3FlexT.Threshold,
        label: string
    ) {
        const instantiate: Cw3FlexT.InstantiateMsg = {
            executor: null,
            group_addr: cw4Addr,
            max_voting_period: maxVotingPeriod,
            threshold: threshold,
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
