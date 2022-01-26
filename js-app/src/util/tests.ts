import { SigningCosmWasmClient, SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate";
import {
  DirectSecp256k1Wallet,
  DirectSecp256k1HdWallet,
} from "@cosmjs/proto-signing";
import { Secp256k1, sha256 } from "@cosmjs/crypto";
import { toBase64, toUtf8, toHex, fromHex } from "@cosmjs/encoding";
import { Coin, calculateFee, GasPrice } from "@cosmjs/stargate";
import * as fs from 'fs';
import { rpcEndPoint } from "./config";

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

export async function createSigningClient(mnemonic: string, addrPrefix: string): Promise<SigningCosmWasmClient>{
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: addrPrefix,
    });
    return SigningCosmWasmClient.connectWithSigner(
      rpcEndPoint!,
      wallet,
      {
        ...defaultSigningClientOptions,
        prefix: addrPrefix!,
      }
    );
}

export async function createSigningClientFromKey(key: Uint8Array, addrPrefix: string): Promise<SigningCosmWasmClient>{
    const wallet = await DirectSecp256k1Wallet.fromKey(
      key,
      addrPrefix!
    );
    return await SigningCosmWasmClient.connectWithSigner(
      rpcEndPoint!,
      wallet,
      {
        ...defaultSigningClientOptions,
        prefix: addrPrefix!,
      }
    );
}

export async function createRelayTransaction(privkey: Uint8Array, nonce: number, jsonMsg: string): Promise<RelayTransaction>{
	const keypair = await Secp256k1.makeKeypair(privkey);
		
	const messageNonceBytes = new Uint8Array([
		...toUtf8(jsonMsg),
		...longToByteArray(nonce),
	]);
	const messageHash = sha256(messageNonceBytes);
	const signature = (
		await Secp256k1.createSignature(messageHash, keypair.privkey)
	).toFixedLength();
	return {
		transaction: {
            user_pubkey: toBase64(keypair.pubkey),
            message: toBase64(toUtf8(jsonMsg)),
            signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
            nonce,
		}
	}
}

export const defaultGasPrice = GasPrice.fromString("0.025ucosm");
export const defaultUploadFee = calculateFee(2_500_000, defaultGasPrice);
export const defaultInstantiateFee = calculateFee(500_000, defaultGasPrice);
export const defaultSendFee = calculateFee(80_000, defaultGasPrice);
export const defaultExecuteFee = calculateFee(200_000, defaultGasPrice);
export const defaultRelayFee = calculateFee(400_000, defaultGasPrice);
export const defaultWalletCreationFee = calculateFee(500_000, defaultGasPrice);
export const defaultMigrateFee = calculateFee(200_000, defaultGasPrice);
export const defaultUpdateAdminFee = calculateFee(80_000, defaultGasPrice);
export const defaultClearAdminFee = calculateFee(80_000, defaultGasPrice);

export interface FactoryInstance {
  readonly instantiateMsg: {
    readonly proxy_code_id: number;
    readonly proxy_multisig_code_id: number;
  };
  readonly address: string;
  readonly initialFund: Coin[];
}

export interface MultisigInstance {
  readonly address: string;
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
	}
}

