import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC, RemoteTunnelT } from "../interfaces";
import { toCosmosMsg } from "../utils/enconding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Vote } from "@dao-dao/types/contracts/cw-proposal-single";
import { Coin } from "../interfaces/Factory.types";

class RemoteProxyClient extends ProxyC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    async executeWasm(
        contractAddr: string,
        msg: unknown,
        funds: Coin[] = [],
        sendFunds: boolean = true,
        memo?: string
    ) {
        return await this.execute(
            {
                msgs: [
                    {
                        wasm: {
                            execute: {
                                contract_addr: contractAddr,
                                funds,
                                msg: toCosmosMsg(msg),
                            },
                        },
                    },
                ],
            },
            "auto",
            memo,
            sendFunds ? funds : []
        );
    }

    async mintGovec(factoryAddr: string, fee: Coin): Promise<ExecuteResult> {
        return await this.executeWasm(factoryAddr, { claim_govec: {} }, [fee]);
    }

    async stakeGovec(tunnelAddr: string, stakingAddr: string, amount: string): Promise<ExecuteResult> {
        const msg: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: {
                    govec_actions: {
                        send: {
                            amount,
                            contract: stakingAddr,
                            msg: toCosmosMsg({ stake: {} }),
                        },
                    },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg);
    }

    async unstakeGovec(tunnelAddr: string, amount: string): Promise<ExecuteResult> {
        const msg: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: {
                    stake_actions: { unstake: { amount } },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg);
    }

    async burnGovec(govecAddr: string): Promise<ExecuteResult> {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: govecAddr,
                            funds: [],
                            msg: toCosmosMsg({ burn: {} }),
                        },
                    },
                },
            ],
        });
    }

    async createProposal(proposalAddr: string, title: string, description: string, msgs: unknown[]) {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: proposalAddr,
                            funds: [],
                            msg: toCosmosMsg({
                                propose: {
                                    description,
                                    latest: null,
                                    msgs,
                                    title,
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

    async sendIbcTokens(tunnelAddr: string, reciverAddr: string, connectId: string, funds: Coin[]) {
        const msg: RemoteTunnelT.ExecuteMsg = {
            ibc_transfer: {
                receiver: {
                    addr: reciverAddr,
                    connection_id: connectId,
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg, funds);
    }
}

export default RemoteProxyClient;
