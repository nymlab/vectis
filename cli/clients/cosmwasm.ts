import { Secp256k1, Secp256k1Keypair, sha256, EnglishMnemonic, Slip10, Slip10Curve, Bip39 } from "@cosmjs/crypto";
import {
    SigningCosmWasmClient,
    UploadResult,
    Code,
    SigningCosmWasmClientOptions,
    ExecuteResult,
} from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet, GeneratedType, OfflineSigner, Registry } from "@cosmjs/proto-signing";
import { Tendermint34Client } from "@cosmjs/tendermint-rpc";
import { makeCosmoshubPath, StdFee } from "@cosmjs/amino";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { AminoTypes, GasPrice } from "@cosmjs/stargate";
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

import {
    coreCodePaths,
    pluginCodePaths,
    cw3FixedMulDownloadLink,
    cw3FlexMulDownloadLink,
    cw4GroupDownloadLink,
    contractsFileNames,
    hostChain,
    hostAccounts,
    hostChainName,
} from "../utils/constants";
import { downloadContract, getContract } from "../utils/fs";
import { longToByteArray } from "../utils/enconding";
import CODES from "../config/onchain-codes.json";

import type { ProxyT, FactoryT } from "../interfaces";
import type { HubContractsUploadResult } from "../interfaces/contracts";
import type { Accounts, Account } from "../config/accounts";
import type { Chain } from "../config/chains";

class CWClient extends SigningCosmWasmClient {
    constructor(
        tmClient: Tendermint34Client | undefined,
        readonly sender: string,
        signer: OfflineSigner,
        options: SigningCosmWasmClientOptions
    ) {
        super(tmClient, signer, options);
    }

    static async generateRandomAccount(prefix: string) {
        return await DirectSecp256k1HdWallet.generate(12, {
            prefix,
        });
    }

    static async connectHostWithAccount(account: Accounts) {
        const acc = hostAccounts[account] as Account;
        return await this.connectWithAccount(hostChain, acc);
    }

    static async createRelayTransaction(
        mnemonic: string,
        nonce: number,
        jsonMsg: string
    ): Promise<ProxyT.RelayTransaction> {
        const keypair = await CWClient.mnemonicToKeyPair(mnemonic);
        const messageNonceBytes = new Uint8Array([...toUtf8(jsonMsg), ...longToByteArray(nonce)]);
        const messageHash = sha256(messageNonceBytes);
        const signature = (await Secp256k1.createSignature(messageHash, keypair.privkey)).toFixedLength();
        return {
            controller_pubkey: toBase64(keypair.pubkey),
            message: toBase64(toUtf8(jsonMsg)),
            signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
            nonce,
        };
    }

    static getContractAddrFromResult(result: ExecuteResult, instMsg: string): string {
        const events = result.logs[0].events; // Wasm event is always the last
        const event = events.find((e) => e.type == "instantiate");
        const factoryEvent = event!.attributes.find((ele) => ele.key == instMsg);
        return factoryEvent?.value as string;
    }

    static getContractAddrFromEvent(result: ExecuteResult, eventType: string, attr: string): string {
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
        const { id, creator, checksum } = await this.getCodeDetails(codeId);
        return { id, creator, checksum };
    }

    async uploadContract(
        codePath: string,
        senderAddress?: FactoryT.Addr,
        uploadFee: StdFee | number | "auto" = "auto"
    ): Promise<UploadResult> {
        const code = getContract(codePath);
        return this.upload(senderAddress ?? this.sender, code, uploadFee);
    }

    /**
     * Uploads contracts needed for e2e tests on local nodes on the hub
     */
    async uploadHubContracts(): Promise<HubContractsUploadResult> {
        const factory = await this.uploadContract(coreCodePaths.factoryCodePath);
        const proxy = await this.uploadContract(coreCodePaths.proxyCodePath);
        const cw3Fixed = await this.uploadContract(coreCodePaths.cw3FixedCodePath);
        const cw3Flex = await this.uploadContract(coreCodePaths.cw3FlexCodePath);
        const cw4Group = await this.uploadContract(coreCodePaths.cw4GroupCodePath);
        const pluginReg = await this.uploadContract(coreCodePaths.pluginRegCodePath);
        const pluginsRes: { [key: string]: UploadResult } = {};

        if (hostChain.plugins) {
            for (let plugin of hostChain.plugins) {
                let result = await this.uploadContract(pluginCodePaths[`${plugin}CodePath`]);
                pluginsRes[plugin] = result;
            }
        }

        return {
            factory,
            proxy,
            cw3Fixed,
            cw3Flex,
            cw4Group,
            pluginReg,
            plugins: pluginsRes,
        };
    }

    private static async connectWithAccount(chain: Chain, { mnemonic }: Account) {
        const { addressPrefix, rpcUrl, gasPrice, feeToken, chainId, estimatedBlockTime } = chain;

        const signer = await CWClient.getSignerWithMnemonic(chain, mnemonic);
        const [{ address }] = await signer.getAccounts();

        const tmClient = await Tendermint34Client.connect(rpcUrl);

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

        return new CWClient(tmClient, address, signer, {
            broadcastPollIntervalMs: 500,
            gasPrice: GasPrice.fromString(gasPrice + feeToken),
            prefix: addressPrefix,
            ...extraOptions,
        });
    }

    static async connectWithOsmo(chain: Chain, signer: any) {
        const { addressPrefix, rpcUrl, gasPrice, feeToken } = chain;
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

        const registry = new Registry(protoRegistry);
        const aminoTypes = new AminoTypes(aminoConverters);

        const stargateClient = await SigningCosmWasmClient.connectWithSigner(rpcUrl, signer, {
            broadcastPollIntervalMs: 300,
            broadcastTimeoutMs: 8_000,
            gasPrice: GasPrice.fromString(gasPrice + feeToken),
            prefix: addressPrefix,
            registry,
            aminoTypes,
        });

        return stargateClient;
    }
}

export default CWClient;

export async function downloadContracts() {
    // Download CwPlus Contracts
    await downloadContract(cw3FixedMulDownloadLink, contractsFileNames.cw3_mutltisig);
    await downloadContract(cw3FlexMulDownloadLink, contractsFileNames.cw3_flex_mutltisig);
    await downloadContract(cw4GroupDownloadLink, contractsFileNames.cw4_group);
}
