import { IbcClient, Link } from "@confio/relayer";
import { GasPrice } from "@cosmjs/stargate";

import CWClient from "./cosmwasm";
import * as CHAINS from "../config/chains";

import type { Chains } from "../config/chains";

class RelayerClient {
    hostChainName: Chains;
    remoteChainName: Chains;
    link: Link | null;
    constructor(hostChainName: Chains, remoteChainName: Chains) {
        this.hostChainName = hostChainName;
        this.remoteChainName = remoteChainName;
        this.link = null;
    }

    get connections(): { hostConnection: string; remoteConnection: string } {
        if (!this.link) throw new Error("Link not initialized");

        return {
            hostConnection: this.link.endA.connectionID,
            remoteConnection: this.link.endB.connectionID,
        };
    }

    async createConnection() {
        const hostClient = await this.createIbcClient(this.hostChainName);
        const remoteClient = await this.createIbcClient(this.remoteChainName);

        this.link = await Link.createWithNewConnections(hostClient, remoteClient);
    }

    async createIbcClient(chainName: Chains): Promise<IbcClient> {
        const signer = await CWClient.getSignerWithAccount(chainName, "admin");
        const [{ address }] = await signer.getAccounts();

        const { rpcUrl, feeToken, gasPrice, estimatedBlockTime, estimatedIndexerTime } = CHAINS[chainName];

        return await IbcClient.connectWithSigner(rpcUrl, signer, address, {
            gasPrice: GasPrice.fromString(gasPrice + feeToken) as any,
            estimatedBlockTime,
            estimatedIndexerTime,
        });
    }

    async recoverConnection() {
        const hostClient = await this.createIbcClient(this.hostChainName);
        const remoteClient = await this.createIbcClient(this.remoteChainName);
        // TODO: Recover from json file
        this.link = await Link.createWithExistingConnections(hostClient, remoteClient, "A", "B");
    }
}

export default RelayerClient;
