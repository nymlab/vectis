import CWClient from "./cosmwasm";
import { Cw3FlexClient as Cw3FlexC, Cw3FlexT } from "../interfaces";
import { FactoryT } from "../interfaces";
import { toCosmosMsg } from "../utils/enconding";
import { maxVotingPeriod, vectisCommitteeThreshold } from "../config/vectis";

class Cw3FlexClient extends Cw3FlexC {
    constructor(cw: CWClient, sender: string, contractAddr: string) {
        super(cw.client, sender, contractAddr);
    }

    static async instantiate(cw: CWClient, codeId: number, cw4Addr: string, label: string) {
        const instantiate: Cw3FlexT.InstantiateMsg = {
            executor: null,
            group_addr: cw4Addr,
            max_voting_period: maxVotingPeriod,
            threshold: vectisCommitteeThreshold,
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

        return new Cw3FlexClient(cw, cw.sender, contractAddress);
    }

    async update_supported_chain(chain_id: string, chain_connection: FactoryT.ChainConnection, factoryAddr: string) {
        let updateMsg: FactoryT.FactoryManagementTraitExecMsg = {
            update_supported_interchain: { chain_id, chain_connection },
        };
        await this.propose(
            {
                description: "update supported chain",
                latest: undefined,
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: factoryAddr,
                                funds: [],
                                msg: toCosmosMsg(updateMsg),
                            },
                        },
                    },
                ],
                title: "update supported chain",
            },
            "auto"
        );
        let proposals = await this.listProposals({});
        const propId = proposals.proposals.pop()!.id;
        await this.execute({ proposalId: propId });
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

    async migrate(contract_addr: string, new_code_id: number) {
        let migrate_msg: Cw3FlexT.WasmMsg = {
            migrate: { contract_addr, msg: toCosmosMsg({ migrate_with_new_state: {} }), new_code_id },
        };
        await this.propose(
            {
                description: "migrate",
                latest: undefined,
                msgs: [
                    {
                        wasm: migrate_msg,
                    },
                ],
                title: "migrate",
            },
            "auto"
        );
        let proposals = await this.listProposals({});
        const propId = proposals.proposals.pop()!.id;
        return this.execute({ proposalId: propId });
    }
}

export default Cw3FlexClient;
