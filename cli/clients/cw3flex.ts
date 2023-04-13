import CWClient from "./cosmwasm";
import { Cw3FlexClient as Cw3FlexC, Cw3FlexT } from "../interfaces";
import { toCosmosMsg } from "../utils/enconding";

// Proposal for the Hub multisig config
// Length of  max Voting Period, Time in seconds
export const maxVotingPeriod: Cw3FlexT.Duration = {
    time: 60 * 60 * 24 * 14,
};

// Vectis Committee Config
// Responsible for approving plugins into the Plugin registry
export const vectisCommitteeThreshold: Cw3FlexT.Threshold = {
    absolute_percentage: { percentage: "0.5" },
};
export const vectisCommittee1Weight: number = 50;
export const vectisCommittee2Weight: number = 50;
export const vectisTechCommittee1Weight: number = 50;
export const vectisTechCommittee2Weight: number = 50;

class Cw3FlexClient extends Cw3FlexC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    static async instantiate(client: CWClient, codeId: number, cw4Addr: string, label: string) {
        const instantiate: Cw3FlexT.InstantiateMsg = {
            executor: null,
            group_addr: cw4Addr,
            max_voting_period: maxVotingPeriod,
            threshold: vectisCommitteeThreshold,
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

    async add_item(key: string, value: string) {
        let add_item_msg: Cw3FlexT.ExecuteMsg = { update_item: { key, value } };
        await this.propose(
            {
                description: "add_item",
                latest: undefined,
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: this.contractAddress,
                                funds: [],
                                msg: toCosmosMsg(add_item_msg),
                            },
                        },
                    },
                ],
                title: "add-item",
            },
            "auto"
        );
        let proposals = await this.listProposals({});
        const propId = proposals.proposals.pop()!.id;
        await this.execute({ proposalId: propId });
    }
}

export default Cw3FlexClient;
