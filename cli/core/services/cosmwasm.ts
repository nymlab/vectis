import {
    SigningCosmWasmClient,
    SigningCosmWasmClientOptions,
    UploadResult,
    CosmWasmClient,
    Code,
    CodeDetails,
} from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1Wallet, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { Secp256k1, Secp256k1Keypair, sha256, EnglishMnemonic, Slip10, Slip10Curve, Bip39 } from "@cosmjs/crypto";
import { makeCosmoshubPath, StdFee } from "@cosmjs/amino";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { defaultGasPrice, defaultUploadFee } from "../utils/fee";
import {
    factoryCodePath,
    adminAddr,
    proxyCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    stakingCodePath,
    daoCodePath,
    voteCodePath,
    proposalSingleCodePath,
    rpcEndPoint,
    cw3FixedMulDownloadLink,
    cw20BaseDownloadLink,
} from "../utils/constants";
import { longToByteArray } from "../utils/enconding";
import { RelayTransaction } from "@vectis/types/contracts/ProxyContract";
import { Addr } from "@vectis/types/contracts/FactoryContract";
import { downloadContract, getContract } from "../utils/fs";
import { DaoDaoContracts } from "core/interfaces/dao";
import codes from "../config/onchain-codes.json";

export const defaultSigningClientOptions: SigningCosmWasmClientOptions = {
    broadcastPollIntervalMs: 300,
    broadcastTimeoutMs: 8_000,
    gasPrice: defaultGasPrice,
};

export async function createSigningClient(mnemonic: string, addrPrefix: string): Promise<SigningCosmWasmClient> {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
        prefix: addrPrefix,
    });
    return SigningCosmWasmClient.connectWithSigner(rpcEndPoint!, wallet, {
        ...defaultSigningClientOptions,
        prefix: addrPrefix!,
    });
}

export async function createSigningClientFromKey(key: Uint8Array, addrPrefix: string): Promise<SigningCosmWasmClient> {
    const wallet = await DirectSecp256k1Wallet.fromKey(key, addrPrefix!);
    return await SigningCosmWasmClient.connectWithSigner(rpcEndPoint!, wallet, {
        ...defaultSigningClientOptions,
        prefix: addrPrefix!,
    });
}

export async function mnemonicToKeyPair(mnemonic: string): Promise<Secp256k1Keypair> {
    const m = new EnglishMnemonic(mnemonic);
    const seed = await Bip39.mnemonicToSeed(m);
    const { privkey } = Slip10.derivePath(Slip10Curve.Secp256k1, seed, makeCosmoshubPath(0));
    return await Secp256k1.makeKeypair(privkey);
}

export async function createRelayTransaction(
    mnemonic: string,
    nonce: number,
    jsonMsg: string
): Promise<RelayTransaction> {
    const keypair = await mnemonicToKeyPair(mnemonic);
    const messageNonceBytes = new Uint8Array([...toUtf8(jsonMsg), ...longToByteArray(nonce)]);
    const messageHash = sha256(messageNonceBytes);
    const signature = (await Secp256k1.createSignature(messageHash, keypair.privkey)).toFixedLength();
    return {
        user_pubkey: toBase64(keypair.pubkey),
        message: toBase64(toUtf8(jsonMsg)),
        signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
        nonce,
    };
}

export async function uploadContract(
    client: SigningCosmWasmClient,
    codePath: string,
    senderAddress?: Addr,
    uploadFee: StdFee | number | "auto" = defaultUploadFee
): Promise<UploadResult> {
    const code = getContract(codePath);
    return client.upload(senderAddress ?? adminAddr, code, uploadFee);
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
 * @param client Signing client
 */
export async function uploadContracts(client: SigningCosmWasmClient): Promise<{
    factoryRes: UploadResult;
    proxyRes: UploadResult;
    multisigRes: UploadResult;
    govecRes: UploadResult;
    stakingRes: Code;
    daoRes: Code;
    voteRes: Code;
    proposalSingleRes: Code;
}> {
    // Upload required contracts
    const factoryRes = await uploadContract(client, factoryCodePath);
    const proxyRes = await uploadContract(client, proxyCodePath);
    const multisigRes = await uploadContract(client, fixMultiSigCodePath);
    const govecRes = await uploadContract(client, govecCodePath);

    const codesObj: { [key: string]: DaoDaoContracts } = codes;

    const network = Object.keys(codesObj).find((ele) => ele == process.env.NETWORK);

    let codeDetails = {} as DaoDaoContracts;
    let codeRes = {} as DaoDaoContracts;

    if (network) {
        codeRes["staking"] = await getOnchainContracts(client, codesObj[network]["staking"].id);
        codeRes["dao"] = await getOnchainContracts(client, codesObj[network]["dao"].id);
        codeRes["vote"] = await getOnchainContracts(client, codesObj[network]["vote"].id);
        codeRes["proposalSingle"] = await getOnchainContracts(client, codesObj[network]["proposalSingle"].id);
    } else {
        codeDetails["staking"] = await uploadContract(client, stakingCodePath);
        codeDetails["dao"] = await uploadContract(client, daoCodePath);
        codeDetails["vote"] = await uploadContract(client, voteCodePath);
        codeDetails["proposalSingle"] = await uploadContract(client, proposalSingleCodePath);

        Object.keys(codeDetails).forEach(async (v) => {
            codeRes[v] = {
                id: codeDetails[v].codeId,
                creator: "",
                checksum: codeDetails[v].originalChecksum,
            } as Code;
        });
    }

    return {
        factoryRes,
        proxyRes,
        multisigRes,
        govecRes,
        stakingRes: codeRes.staking,
        daoRes: codeRes.dao,
        voteRes: codeRes.vote,
        proposalSingleRes: codeRes.proposalSingle,
    };
}

export async function getOnchainContracts(client: SigningCosmWasmClient, codeId: number): Promise<Code> {
    const res = await client.getCodeDetails(codeId);
    return {
        id: res.id,
        creator: res.creator,
        checksum: res.checksum,
    };
}

export async function downloadContracts() {
    //TODO: add downloads for dao-contract v1
    await downloadContract(cw3FixedMulDownloadLink, "cw3_fixed_multisig.wasm");
    await downloadContract(cw20BaseDownloadLink, "cw20_base.wasm");
}
