import { SigningCosmWasmClient, SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1Wallet, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { Secp256k1, Secp256k1Keypair, sha256, EnglishMnemonic, Slip10, Slip10Curve, Bip39 } from "@cosmjs/crypto";
import { makeCosmoshubPath } from "@cosmjs/amino";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { Coin, calculateFee, GasPrice, coin } from "@cosmjs/stargate";
import { rpcEndPoint, gasprice, coinMinDenom } from "./config";
import * as fs from "fs";

export const defaultSigningClientOptions: SigningCosmWasmClientOptions = {
    broadcastPollIntervalMs: 300,
    broadcastTimeoutMs: 8_000,
};

export function getContract(path: string): Uint8Array {
    return fs.readFileSync(path);
}

/// Big endian
export function longToByteArray(long: number): Uint8Array {
    // we want to represent the input as a 8-bytes array
    var byteArray = new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0]);

    for (var index = byteArray.length - 1; index >= 0; index--) {
        var byte = long & 0xff;
        byteArray[index] = byte;
        long = (long - byte) / 256;
    }

    return byteArray;
}

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
        transaction: {
            user_pubkey: toBase64(keypair.pubkey),
            message: toBase64(toUtf8(jsonMsg)),
            signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
            nonce,
        },
    };
}

export const defaultGasPrice = GasPrice.fromString(gasprice!);
export const defaultUploadFee = calculateFee(55_500_000, defaultGasPrice);
export const defaultInstantiateFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultSendFee = calculateFee(800_000, defaultGasPrice);
export const defaultExecuteFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultRelayFee = calculateFee(1_400_000, defaultGasPrice);
export const defaultWalletCreationFee = calculateFee(1_500_000, defaultGasPrice);
export const defaultMigrateFee = calculateFee(1_200_000, defaultGasPrice);
export const defaultUpdateAdminFee = calculateFee(800_000, defaultGasPrice);
export const defaultClearAdminFee = calculateFee(800_000, defaultGasPrice);
export const walletFee = coin(100, coinMinDenom!);

export interface FactoryInstance {
    readonly instantiateMsg: {
        readonly proxy_code_id: number;
        readonly proxy_multisig_code_id: number;
        readonly addr_prefix: string;
    };
    readonly address: string;
    readonly initialFund: Coin[];
}

export interface MultisigInstance {
    readonly address: string;
}

export interface DAOInstance {
    readonly address: string;
}

export interface GovecInstance {
    readonly address: string | null;
    readonly codeId: number;
}

export interface StakingInstance {
    readonly address: string | null;
    readonly codeId: number;
}

export interface CreateWalletMsg {
    user_pubkey: string;
    guardians: {
        addresses: string[];
        guardians_multisig: MultiSig | null;
    };
    relayers: string[];
    proxy_initial_funds: Coin[];
}

export interface CreateGovernanceMsg {
    staking_options: StakingOptions | null;
    initial_balances: Cw20Coin[];
}

export interface StakingOptions {
    duration: Height | Time | null;
    code_id: number;
}

export interface Height {
    height: number;
}

export interface Time {
    time: number;
}

export interface Cw20Coin {
    address: string;
    amount: number;
}

export interface MultiSig {
    threshold_absolute_count: number;
    multisig_initial_funds: Coin[];
}

export interface WalletInstance {
    readonly address: string;
    readonly instantiateMsg: CreateWalletMsg;
}

export interface BankMsg {
    readonly bank: {
        send: {
            readonly to_address: string;
            readonly amount: readonly Coin[];
        };
    };
}

export interface WasmExecuteMsg {
    readonly wasm: {
        readonly execute: {
            readonly contract_addr: string;
            readonly msg: any;
            readonly funds: readonly Coin[];
        };
    };
}

export interface RelayTransaction {
    transaction: {
        user_pubkey: string;
        message: string;
        signature: string;
        nonce: number;
    };
}
