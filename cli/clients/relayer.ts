import crypto from "crypto";
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
import { loadIbcInfo, writeInCacheFolder } from "../utils/fs";

class RelayerClient {
    link: Link | null;
    wasmChannel: ChannelPair | null;
    transferChannel: ChannelPair | null;
    constructor() {
        this.link = null;
        this.wasmChannel = null;
        this.transferChannel = null;
    }

    get connections(): { hostConnection: string; remoteConnection: string } {
        if (!this.link) throw new Error("Link not initialized");

        return {
            hostConnection: this.link.endA.connectionID,
            remoteConnection: this.link.endB.connectionID,
        };
    }

    get channels(): { wasm: ChannelPair | null; transfer: ChannelPair | null } {
        return {
            wasm: this.wasmChannel,
            transfer: this.transferChannel,
        };
    }

    get denoms(): { src: string; dest: string } {
        return {
            src: this.getDenomIBC("transfer", this.transferChannel?.src.channelId as string, hostChain.feeToken),
            dest: this.getDenomIBC("transfer", this.transferChannel?.dest.channelId as string, remoteChain.feeToken),
        };
    }

    backupConnection() {
        if (!this.link) throw new Error("Link not initialized");

        const relayerConfig = {
            src: this.link.endA.connectionID,
            dest: this.link.endB.connectionID,
        };

        writeInCacheFolder("ibcInfo.json", JSON.stringify(relayerConfig, null, 2));
    }

    getDenomIBC(port: string, channel: string, token: string) {
        return "ibc/" + crypto.createHash("sha256").update(`${port}/${channel}/${token}`).digest("hex").toUpperCase();
    }

    async backupChannels() {
        const ibcInfo = loadIbcInfo();
        writeInCacheFolder("ibcInfo.json", JSON.stringify({ ...ibcInfo, channels: this.channels }, null, 2));
    }

    async relayAll() {
        if (!this.link) throw new Error("Link not initialized");
        return await this.link.relayAll();
    }

    async connect() {
        const ibcInfo = loadIbcInfo();

        const hostConnections = connections[hostChainName as keyof typeof connections] || {};
        const connectionsId = hostConnections[remoteChainName as keyof typeof hostConnections] as
            | { src: string; dest: string }
            | undefined;

        if (ibcInfo) {
            try {
                return await this.recoverConnection(ibcInfo.src, ibcInfo.dest);
            } catch (err) {
                return this.createConnection();
            }
        } else if (connectionsId) {
            return await this.recoverConnection(connectionsId.src, connectionsId.dest);
        }

        return this.createConnection();
    }

    async loadChannels() {
        const ibcInfo = loadIbcInfo();
        const hostConnections = connections[hostChainName as keyof typeof connections] || {};
        const connectionsInfo = hostConnections[remoteChainName as keyof typeof hostConnections] as
            | {
                  channels: {
                      wasm: ChannelPair | undefined;
                      transfer: ChannelPair | undefined;
                  };
              }
            | undefined;

        this.wasmChannel = ibcInfo?.channels?.wasm || connectionsInfo?.channels?.wasm || null;
        this.transferChannel = ibcInfo?.channels?.transfer || connectionsInfo?.channels?.transfer || null;

        return this.channels;
    }

    async createConnection() {
        const hostClient = await this.createIbcClient(hostChain, hostAccounts.admin.mnemonic as string);
        const remoteClient = await this.createIbcClient(remoteChain, remoteAccounts.admin.mnemonic as string);

        this.link = await Link.createWithNewConnections(hostClient, remoteClient);
        this.backupConnection();
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

    async createChannel(src: string, dest: string, version: string) {
        if (!this.link) throw new Error("Link not initialized");

        if (version === "ics20-1") {
            this.transferChannel = await this.link.createChannel("A", src, dest, 1, version);
        } else if (version === "vectis-v1") {
            this.wasmChannel = await this.link.createChannel("A", src, dest, 1, version);
        }

        await this.backupChannels();

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
