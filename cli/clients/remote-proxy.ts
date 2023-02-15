import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC, RemoteTunnelT } from "../interfaces";
import {
    ProposalSingleExecuteMsg as ProposalPackageMsg,
    ExecuteMsgForProposeMessageAndExecuteExt as PreProposalPackageMsg,
} from "../interfaces/RemoteTunnel.types";
import { toCosmosMsg } from "../utils/enconding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin } from "../interfaces/Factory.types";
import { CosmosMsgForEmpty } from "../interfaces/Cw3Flex.types";

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

    async executeProposalAction(tunnelAddr: string, proposalAddr: string, msg: ProposalPackageMsg) {
        let packetMsg: RemoteTunnelT.RemoteTunnelPacketMsg = {
            proposal_actions: {
                prop_module_addr: proposalAddr,
                msg,
            },
        };
        const action: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: packetMsg,
            },
        };
        return await this.executeWasm(tunnelAddr, action);
    }

    async executePreProposalAction(tunnelAddr: string, preProposalAddr: string, msg: PreProposalPackageMsg) {
        let packetMsg: RemoteTunnelT.RemoteTunnelPacketMsg = {
            pre_proposal_actions: {
                pre_prop_module_addr: preProposalAddr,
                msg,
            },
        };
        const action: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: packetMsg,
            },
        };

        console.log("\n executePreProposalAction, ", action);
        return await this.executeWasm(tunnelAddr, action);
    }

    // These might not be working, why is there no relay_from params
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
                            relayed_from: this.contractAddress,
                        },
                    },
                },
            },
        };
        console.log("remote proxy client:::", msg);
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

    async exitGovec(tunnelAddr: string): Promise<ExecuteResult> {
        const msg: RemoteTunnelT.ExecuteMsg = {
            dao_actions: {
                msg: {
                    govec_actions: { exit: {} },
                },
            },
        };
        return await this.executeWasm(tunnelAddr, msg);
    }

    async createPreProposal(
        tunnelAddr: string,
        preProposalAddr: string,
        title: string,
        description: string,
        msgs: CosmosMsgForEmpty[]
    ) {
        return await this.executePreProposalAction(tunnelAddr, preProposalAddr, {
            propose: {
                msg: {
                    propose: {
                        description,
                        msgs,
                        title,
                        relayed_from: "somestring",
                    },
                },
            },
        });
    }

    async voteProposal(tunnelAddr: string, proposalAddr: string, proposalId: number, vote: RemoteTunnelT.Vote) {
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
