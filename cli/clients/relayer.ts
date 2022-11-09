import { ChannelPair } from "@confio/relayer/build/lib/link";
import { IbcClient, Link } from "@confio/relayer";
import { GasPrice } from "@cosmjs/stargate";

import CWClient from "./cosmwasm";
import {
    hostAccounts,
    hostChain,
    hostChainName,
    remoteAccounts,
    remoteChain,
    remoteChainName,
} from "../utils/constants";

import type { Chain } from "../config/chains";
import connections from "../config/relayer/connections.json";
import { writeRelayerConfig } from "../utils/fs";

class RelayerClient {
    link: Link | null;
    channel: ChannelPair | null;
    constructor() {
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

    backup() {
        if (!this.link) throw new Error("Link not initialized");
        const relayerConfig = {
            [hostChainName]: {
                [remoteChainName]: {
                    src: this.link.endA.connectionID,
                    dest: this.link.endB.connectionID,
                },
            },
        };

        writeRelayerConfig(Object.assign(connections, relayerConfig), "connections.json");
    }

    async relayAll() {
        if (!this.link) throw new Error("Link not initialized");
        return await this.link.relayAll();
    }

    async connect() {
        const hostConnections = connections[hostChainName as keyof typeof connections] || {};
        const connectionsId = hostConnections[remoteChainName as keyof typeof hostConnections] as
            | { src: string; dest: string }
            | undefined;
        return connectionsId ? this.recoverConnection(connectionsId.src, connectionsId.dest) : this.createConnection();
    }

    async createConnection() {
        const hostClient = await this.createIbcClient(hostChain, hostAccounts.admin.mnemonic as string);
        const remoteClient = await this.createIbcClient(remoteChain, remoteAccounts.admin.mnemonic as string);

        this.link = await Link.createWithNewConnections(hostClient, remoteClient);
        this.backup();
        return this.connections;
    }

    async createIbcClient(chain: Chain, mnemonic: string): Promise<IbcClient> {
        const signer = await CWClient.getSignerWithMnemonic(chain, mnemonic);
        const [{ address }] = await signer.getAccounts();

        const { rpcUrl, feeToken, gasPrice, estimatedBlockTime, estimatedIndexerTime } = chain;

        return await IbcClient.connectWithSigner(rpcUrl, signer, address, {
            gasPrice: GasPrice.fromString(gasPrice + feeToken),
            estimatedBlockTime,
            estimatedIndexerTime,
        });
    }

    async createChannel(src: string, dest: string) {
        if (!this.link) throw new Error("Link not initialized");

        this.channel = await this.link.createChannel("A", src, dest, 1, "vectis-v1");
        return this.channels;
    }

    async recoverConnection(connA: string, connB: string) {
        const hostClient = await this.createIbcClient(hostChain, hostAccounts.admin.mnemonic as string);
        const remoteClient = await this.createIbcClient(remoteChain, remoteAccounts.admin.mnemonic as string);

        this.link = await Link.createWithExistingConnections(hostClient, remoteClient, connA, connB);
        return this.connections;
    }
}

export default RelayerClient;
