import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC } from "../interfaces";
import { toCosmosMsg } from "../utils/enconding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";

class ProxyClient extends ProxyC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    async mintGovec(factoryAddr: string): Promise<ExecuteResult> {
        return await this.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: factoryAddr,
                            funds: [],
                            msg: toCosmosMsg({ claim_govec: {} }),
                        },
                    },
                },
            ],
        });
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
}

export default ProxyClient;
