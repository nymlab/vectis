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

    async executeProposalAction(tunnelAddr: string, proposalAddr: string, msg: Record<string, unknown>) {
        const action = {
            dao_actions: {
                msg: {
                    proposal_actions: {
                        prop_module_addr: proposalAddr,
                        msg,
                    },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, action);
    }

    async mintGovec(factoryAddr: string, fee: Coin): Promise<ExecuteResult> {
        return await this.executeWasm(factoryAddr, { claim_govec: {} }, [fee]);
    }

    async transferGovec(tunnelAddr: string, reciverAddr: string, amount: string) {
        const msg: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: {
                    govec_actions: {
                        transfer: {
                            amount,
                            recipient: reciverAddr,
                        },
                    },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg);
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

    async burnGovec(tunnelAddr: string): Promise<ExecuteResult> {
        const msg: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: {
                    govec_actions: { burn: {} },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg);
    }

    async createProposal(
        tunnelAddr: string,
        proposalAddr: string,
        title: string,
        description: string,
        msgs: unknown[]
    ) {
        return await this.executeProposalAction(tunnelAddr, proposalAddr, {
            propose: {
                title,
                description,
                msgs,
            },
        });
    }

    async voteProposal(tunnelAddr: string, proposalAddr: string, proposalId: number, vote: Vote) {
        return await this.executeProposalAction(tunnelAddr, proposalAddr, {
            vote: {
                proposal_id: proposalId,
                vote,
            },
        });
    }

    async executeProposal(tunnelAddr: string, proposalAddr: string, proposalId: number) {
        return await this.executeProposalAction(tunnelAddr, proposalAddr, {
            execute: {
                proposal_id: proposalId,
            },
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
