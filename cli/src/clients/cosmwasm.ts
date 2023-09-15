import path from "path";
import { Secp256k1, Secp256k1Keypair, EnglishMnemonic, Slip10, Slip10Curve, Bip39 } from "@cosmjs/crypto";
import { SigningArchwayClient } from "@archwayhq/arch3.js";
import { SigningCosmWasmClient, UploadResult, Code, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet, GeneratedType } from "@cosmjs/proto-signing";
import { Tendermint37Client } from "@cosmjs/tendermint-rpc";
import { makeCosmoshubPath, StdFee } from "@cosmjs/amino";
import { GasPrice } from "@cosmjs/stargate";
import {
    cosmosAminoConverters,
    cosmosProtoRegistry,
    cosmwasmAminoConverters,
    cosmwasmProtoRegistry,
    ibcProtoRegistry,
    ibcAminoConverters,
    osmosisAminoConverters,
    osmosisProtoRegistry,
} from "osmojs";

import { getContract } from "../utils/fs";

import type { FactoryT } from "../interfaces";
import type { Accounts, Account } from "../config/accounts";
import * as chainConfigs from "../config/chains";
import type { Chain } from "../config/chains";
import { accountsPath } from "../config/fs";

class CWClient {
    client: SigningArchwayClient | SigningCosmWasmClient;
    readonly sender: string;
    constructor(client: SigningArchwayClient | SigningCosmWasmClient, sender: string) {
        this.client = client;
        this.sender = sender;
    }

    static async generateRandomAccount(prefix: string) {
        return await DirectSecp256k1HdWallet.generate(12, {
            prefix,
        });
    }

    static async connectHostWithAccount(account: Accounts, network: string) {
        const accounts = await import(path.join(accountsPath, `/${network}.json`));
        const chain = chainConfigs[network as keyof typeof chainConfigs] as Chain;

        return await this.connectWithAccount(chain, accounts[account] as Account);
    }

    static getAddrFromInstantianteResult(result: ExecuteResult): string {
        const events = result.logs[0].events; // Wasm event is always the last
        const event = events.find((e) => e.type == "instantiate");
        const factoryEvent = event!.attributes.find((ele) => ele.key == "_contract_address");
        return factoryEvent?.value as string;
    }

    static getEventAttrValue(result: ExecuteResult, eventType: string, attr: string): string {
        let events = result.events;
        const event = events.find((e) => e.type == eventType);
        const attribute = event!.attributes.find((ele) => ele.key == attr);
        return attribute?.value as string;
    }

    static async getSignerWithMnemonic({ addressPrefix }: Chain, mnemonic: string): Promise<DirectSecp256k1HdWallet> {
        return await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
            prefix: addressPrefix,
        });
    }

    static async mnemonicToKeyPair(mnemonic: string): Promise<Secp256k1Keypair> {
        const m = new EnglishMnemonic(mnemonic);
        const seed = await Bip39.mnemonicToSeed(m);
        const { privkey } = Slip10.derivePath(Slip10Curve.Secp256k1, seed, makeCosmoshubPath(0) as any);
        return await Secp256k1.makeKeypair(privkey);
    }

    async getOnchainContracts(codeId: number): Promise<Code> {
        const { id, creator, checksum } = await this.client.getCodeDetails(codeId);
        return { id, creator, checksum };
    }

    async uploadContract(
        codePath: string,
        senderAddress?: FactoryT.Addr,
        uploadFee: StdFee | number | "auto" = "auto"
    ): Promise<UploadResult> {
        const code = getContract(codePath);
        return this.client.upload(senderAddress ?? this.sender, code, uploadFee);
    }

    private static async connectWithAccount(chain: Chain, { mnemonic }: Account) {
        const { addressPrefix, rpcUrl, gasPrice, feeToken, chainId } = chain;

        const signer = await CWClient.getSignerWithMnemonic(chain, mnemonic);
        const [{ address }] = await signer.getAccounts();

        const tmClient = await Tendermint37Client.connect(rpcUrl);

        const protoRegistry: ReadonlyArray<[string, GeneratedType]> = [
            ...cosmosProtoRegistry,
            ...cosmwasmProtoRegistry,
            ...ibcProtoRegistry,
            ...osmosisProtoRegistry,
        ];

        const aminoConverters = {
            ...cosmosAminoConverters,
            ...cosmwasmAminoConverters,
            ...ibcAminoConverters,
            ...osmosisAminoConverters,
        };

        const extraOptions = chainId.includes("osmo") ? { protoRegistry, aminoConverters } : {};

        if (addressPrefix == "archway") {
            console.log("archway client");
            const client = await SigningArchwayClient.createWithSigner(tmClient, signer);
            return new CWClient(client, address);
        } else {
            const client = await SigningCosmWasmClient.createWithSigner(tmClient, signer, {
                broadcastPollIntervalMs: 500,
                gasPrice: GasPrice.fromString(gasPrice + feeToken),
                ...extraOptions,
            });
            return new CWClient(client, address);
        }
    }
}

export default CWClient;
