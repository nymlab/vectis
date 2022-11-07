import { ChannelPair } from "@confio/relayer/build/lib/link";
import { IbcClient, Link } from "@confio/relayer";
import { GasPrice } from "@cosmjs/stargate";

import CWClient from "./cosmwasm";
import * as CHAINS from "../config/chains";

import type { Chains } from "../config/chains";

class RelayerClient {
    hostChainName: Chains;
    remoteChainName: Chains;
    link: Link | null;
    channel: ChannelPair | null;
    constructor(hostChainName: Chains, remoteChainName: Chains) {
        this.hostChainName = hostChainName;
        this.remoteChainName = remoteChainName;
        this.link = null;
        this.channel = null;
    }

    get connections(): { hostConnection: string; remoteConnection: string } {
        if (!this.link) throw new Error("Link not initialized");

        return {
            hostConnection: this.link.endA.connectionID,
            remoteConnection: this.link.endB.connectionID,
        };
    }

    get channels(): { hostChannel: string; remoteChannel: string } {
        if (!this.channel) throw new Error("Channel not initialized");

        return {
            hostChannel: this.channel.src.channelId,
            remoteChannel: this.channel.dest.channelId,
        };
    }

    async relayAll() {
        if (!this.link) throw new Error("Link not initialized");
        return await this.link.relayAll();
    }

    async createConnection() {
        const hostClient = await this.createIbcClient(this.hostChainName);
        const remoteClient = await this.createIbcClient(this.remoteChainName);

        this.link = await Link.createWithNewConnections(hostClient, remoteClient);
        return this.connections;
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

    async createChannel(src: string, dest: string) {
        if (!this.link) throw new Error("Link not initialized");

        this.channel = await this.link.createChannel("A", src, dest, 1, "vectis-v1");
        return this.channels;
    }

    async recoverConnection() {
        const hostClient = await this.createIbcClient(this.hostChainName);
        const remoteClient = await this.createIbcClient(this.remoteChainName);
        // TODO: Recover from json file
        this.link = await Link.createWithExistingConnections(
            hostClient,
            remoteClient,
            "connection-16",
            "connection-18"
        );
    }
}

export default RelayerClient;
