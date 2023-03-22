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
import { AminoTypes, GasPrice, Block } from "@cosmjs/stargate";
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
    codePaths,
    cw3FixedMulDownloadLink,
    cw3FlexMulDownloadLink,
    cw4GroupDownloadLink,
    cw20BaseDownloadLink,
    cwDaoDownloadLink,
    cw20StakingDownloadLink,
    cw20VotingDownloadLink,
    cw20ProposalSingleDownloadLink,
    cwPreProposalSingleDownloadLink,
    contractsFileNames,
    hostChain,
    hostAccounts,
    remoteAccounts,
    remoteChain,
    hostChainName,
} from "../utils/constants";
import { downloadContract, getContract } from "../utils/fs";
import { longToByteArray } from "../utils/enconding";
import CODES from "../config/onchain-codes.json";

import type { ProxyT, FactoryT } from "../interfaces";
import type { DaoDaoContracts, DaoContractsUploadResult, RemoteContractsUploadResult } from "../interfaces/contracts";
import type { Accounts, Account } from "../config/accounts";
import type { Chains, Chain } from "../config/chains";

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

    static async connectRemoteWithAccount(account: Accounts) {
        const acc = remoteAccounts[account] as Account;
        return await this.connectWithAccount(remoteChain, acc);
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

    async getDaoDaoOnChainContracts(codeIds: typeof CODES["juno_testnet"]): Promise<DaoDaoContracts> {
        const dao = await this.getOnchainContracts(codeIds.dao.id);
        return { dao };
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
     * Uploads contracts needed for e2e tests on local nodes
     * Note: dao-contracts do not need to be uploaded on juno-testnet / juno-mainnet
     *
     */
    async uploadHostContracts(): Promise<DaoContractsUploadResult> {
        const daodaoCodes = CODES[hostChainName as keyof typeof CODES];

        const { dao } = daodaoCodes
            ? await this.getDaoDaoOnChainContracts(daodaoCodes)
            : await this.uploadDaoDaoContracts();

        const staking = await this.uploadContract(codePaths.stakingCodePath);
        const vote = await this.uploadContract(codePaths.voteCodePath);
        const proposalSingle = await this.uploadContract(codePaths.proposalSingleCodePath);
        const preProposalSingle = await this.uploadContract(codePaths.preProposalSingleCodePath);
        const daoTunnel = await this.uploadContract(codePaths.daoTunnelCodePath);
        const factory = await this.uploadContract(codePaths.factoryCodePath);
        const proxy = await this.uploadContract(codePaths.proxyCodePath);
        const cw3Fixed = await this.uploadContract(codePaths.cw3FixedCodePath);
        const cw3Flex = await this.uploadContract(codePaths.cw3FlexCodePath);
        const cw4Group = await this.uploadContract(codePaths.cw4GroupCodePath);
        const govec = await this.uploadContract(codePaths.govecCodePath);
        const pluginReg = await this.uploadContract(codePaths.pluginRegCodePath);

        return {
            daoTunnel,
            factory,
            proxy,
            cw3Fixed,
            cw3Flex,
            cw4Group,
            govec,
            pluginReg,
            staking,
            dao,
            vote,
            proposalSingle,
            preProposalSingle,
        };
    }

    async uploadRemoteContracts(): Promise<RemoteContractsUploadResult> {
        const remoteTunnel = await this.uploadContract(codePaths.remoteTunnelCodePath);
        const remoteProxy = await this.uploadContract(codePaths.remoteProxyCodePath);
        const remoteFactory = await this.uploadContract(codePaths.remoteFactoryCodePath);
        const cw3Fixed = await this.uploadContract(codePaths.cw3FixedCodePath);

        return { remoteTunnel, remoteProxy, remoteFactory, cw3Fixed };
    }

    async uploadDaoDaoContracts() {
        const staking = await this.uploadContract(codePaths.stakingCodePath);
        const dao = await this.uploadContract(codePaths.daoCodePath);
        const vote = await this.uploadContract(codePaths.voteCodePath);
        const proposalSingle = await this.uploadContract(codePaths.proposalSingleCodePath);
        const preProposalSingle = await this.uploadContract(codePaths.preProposalSingleCodePath);

        return { staking, dao, vote, proposalSingle, preProposalSingle };
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
    await downloadContract(cw20BaseDownloadLink, contractsFileNames.cw20_base);
    await downloadContract(cw3FlexMulDownloadLink, contractsFileNames.cw3_flex_mutltisig);
    await downloadContract(cw4GroupDownloadLink, contractsFileNames.cw4_group);
    // Download DAODAO Contracts
    await downloadContract(cwDaoDownloadLink, contractsFileNames.cw_dao);
    await downloadContract(cw20StakingDownloadLink, contractsFileNames.cw20_staking);
    await downloadContract(cw20VotingDownloadLink, contractsFileNames.cw20_voting);
    await downloadContract(cw20ProposalSingleDownloadLink, contractsFileNames.cw_proposal_single);
    await downloadContract(cwPreProposalSingleDownloadLink, contractsFileNames.cw_pre_proposal_approval_single);
}
