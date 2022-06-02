import { SigningCosmWasmClient, SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1Wallet, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { Secp256k1, Secp256k1Keypair, sha256, EnglishMnemonic, Slip10, Slip10Curve, Bip39 } from "@cosmjs/crypto";
import { makeCosmoshubPath } from "@cosmjs/amino";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { rpcEndPoint } from "./env";
import { RelayTransaction } from "../../types/ProxyContract";
import { defaultGasPrice } from "./fee";

export const defaultSigningClientOptions: SigningCosmWasmClientOptions = {
    broadcastPollIntervalMs: 300,
    broadcastTimeoutMs: 8_000,
    gasPrice: defaultGasPrice,
};

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
        user_pubkey: toBase64(keypair.pubkey),
        message: toBase64(toUtf8(jsonMsg)),
        signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
        nonce,
    };
}

export const delay = (ms: number) => {
    return new Promise((resolve) => setTimeout(resolve, ms));
};
