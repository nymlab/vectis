import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC } from "../interfaces";
import { toCosmosMsg } from "../utils/enconding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin } from "../interfaces/Factory.types";

class ProxyClient extends ProxyC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    async mintGovec(factoryAddr: string, fee: Coin): Promise<ExecuteResult> {
        return await this.execute(
            {
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: factoryAddr,
                                funds: [fee as Coin],
                                msg: toCosmosMsg({ claim_govec: {} }),
                            },
                        },
                    },
                ],
            },
            "auto",
            undefined,
            [fee]
        );
    }

    async stakeGovec(govecAddr: string, stakingAddr: string, amount: string): Promise<ExecuteResult> {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: govecAddr,
                            funds: [],
                            msg: toCosmosMsg({
                                send: {
                                    amount,
                                    contract: stakingAddr,
                                    msg: toCosmosMsg({ stake: {} }),
                                    relayed_from: undefined,
                                },
                            }),
                        },
                    },
                },
            ],
        });
    }

    async unstakeGovec(stakingAddr: string, amount: string): Promise<ExecuteResult> {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: stakingAddr,
                            funds: [],
                            msg: toCosmosMsg({ unstake: { amount } }),
                        },
                    },
                },
            ],
        });
    }

    async exitGovec(govecAddr: string): Promise<ExecuteResult> {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: govecAddr,
                            funds: [],
                            msg: toCosmosMsg({ exit: {} }),
                        },
                    },
                },
            ],
        });
    }

    async createPreProposal(preproposalAddr: string, title: string, description: string, msgs: unknown[]) {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: preproposalAddr,
                            funds: [],
                            msg: toCosmosMsg({
                                propose: {
                                    msg: {
                                        propose: {
                                            description,
                                            msgs,
                                            title,
                                        },
                                    },
                                },
                            }),
                        },
                    },
                },
            ],
        });
    }

    async voteProposal(proposalAddr: string, proposalId: number, vote: Vote) {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: proposalAddr,
                            funds: [],
                            msg: toCosmosMsg({
                                vote: {
                                    proposal_id: proposalId,
                                    vote,
                                },
                            }),
                        },
                    },
                },
            ],
        });
    }

    async executeProposal(proposalAddr: string, proposalId: number) {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: proposalAddr,
                            funds: [],
                            msg: toCosmosMsg({
                                execute: {
                                    proposal_id: proposalId,
                                },
                            }),
                        },
                    },
                },
            ],
        });
    }
}

export default ProxyClient;
