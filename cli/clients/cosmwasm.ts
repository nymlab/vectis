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
    factoryCodePath,
    proxyCodePath,
    fixMultiSigCodePath,
    cw3FlexCodePath,
    cw4GroupCodePath,
    govecCodePath,
    pluginRegCodePath,
    stakingCodePath,
    daoCodePath,
    voteCodePath,
    proposalSingleCodePath,
    preProposalSingleCodePath,
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
    daoTunnelCodetPath,
    remoteTunnelCodePath,
    remoteProxyCodePath,
    remoteFactoryCodePath,
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
import type { DaoDaoContracts } from "../interfaces/contracts";
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
        const attributes = events[events.length - 1].attributes;
        const factoryEvent = attributes.find((ele) => ele.key == instMsg);
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
        const staking = await this.getOnchainContracts(codeIds.staking.id);
        const dao = await this.getOnchainContracts(codeIds.dao.id);
        const vote = await this.getOnchainContracts(codeIds.vote.id);
        const proposalSingle = await this.getOnchainContracts(codeIds.proposalSingle.id);
        const preProposalSingle = await this.getOnchainContracts(codeIds.preProposalSingle.id);

        return { staking, dao, vote, proposalSingle, preProposalSingle };
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
     *  - factory
     *  - proxy
     *  - cw3 fixed mulitisig
     *  - govec token contract
     *  - dao-contracts: core
     *  - dao-contracts: cw20_stake
     *  - dao-contracts: cw20_staked_balance_voting
     *  - dao-contracts: proposal-single
     *
     *  Note: dao-contracts do not need to be uploaded on juno-testnet / juno-mainnet
     *
     *  Current version of DAO contracts: 60b710df2f3abb8ca275ad16a86ce3b0c265a339 (on testnet)
     *
     */
    async uploadHostContracts(): Promise<{
        daoTunnelRes: UploadResult;
        factoryRes: UploadResult;
        proxyRes: UploadResult;
        multisigRes: UploadResult;
        Cw3FlexRes: UploadResult;
        Cw4GroupRes: UploadResult;
        govecRes: UploadResult;
        pluginRegRes: UploadResult;
        stakingRes: UploadResult | Code;
        daoRes: UploadResult | Code;
        voteRes: UploadResult | Code;
        proposalSingleRes: UploadResult | Code;
        preProposalSingleRes: UploadResult | Code;
    }> {
        const daodaoCodes = CODES[hostChainName as keyof typeof CODES];

        const { staking, dao, vote, proposalSingle, preProposalSingle } = daodaoCodes
            ? await this.getDaoDaoOnChainContracts(daodaoCodes)
            : await this.uploadDaoDaoContracts();

        const daoTunnelRes = await this.uploadContract(daoTunnelCodetPath);
        const factoryRes = await this.uploadContract(factoryCodePath);
        const proxyRes = await this.uploadContract(proxyCodePath);
        const multisigRes = await this.uploadContract(fixMultiSigCodePath);
        const Cw3FlexRes = await this.uploadContract(cw3FlexCodePath);
        const Cw4GroupRes = await this.uploadContract(cw4GroupCodePath);
        const govecRes = await this.uploadContract(govecCodePath);
        const pluginRegRes = await this.uploadContract(pluginRegCodePath);

        return {
            daoTunnelRes,
            factoryRes,
            proxyRes,
            multisigRes,
            Cw3FlexRes,
            Cw4GroupRes,
            govecRes,
            pluginRegRes,
            stakingRes: staking,
            daoRes: dao,
            voteRes: vote,
            proposalSingleRes: proposalSingle,
            preProposalSingleRes: preProposalSingle,
        };
    }

    async uploadRemoteContracts(): Promise<{
        remoteTunnel: UploadResult;
        remoteProxy: UploadResult;
        remoteFactory: UploadResult;
        remoteMultisig: UploadResult;
    }> {
        const remoteTunnel = await this.uploadContract(remoteTunnelCodePath);
        const remoteProxy = await this.uploadContract(remoteProxyCodePath);
        const remoteFactory = await this.uploadContract(remoteFactoryCodePath);
        const remoteMultisig = await this.uploadContract(fixMultiSigCodePath);

        return { remoteTunnel, remoteProxy, remoteFactory, remoteMultisig };
    }

    async uploadDaoDaoContracts() {
        const staking = await this.uploadContract(stakingCodePath);
        const dao = await this.uploadContract(daoCodePath);
        const vote = await this.uploadContract(voteCodePath);
        const proposalSingle = await this.uploadContract(proposalSingleCodePath);
        const preProposalSingle = await this.uploadContract(preProposalSingleCodePath);

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
