import { coin } from "@cosmjs/stargate";
import { NetworkOptions } from "../interfaces/network";
import { getVectisContractPaths, getDownloadContractsPath } from "./fs";
import networks from "./networks";
import wallets from "./accounts.json";

const network = networks[process.env.NETWORK as NetworkOptions];

const accounts = wallets[network.addressPrefix as "juno" | "wasm"];

export const chainId = network.chainId;
export const addrPrefix = network.addressPrefix;
export const rpcEndPoint = network.rpcUrl;
export const gasPrice = network.gasPrice + network.feeToken;
export const coinDenom = network.feeToken;
export const coinMinDenom = network.feeToken;

export const adminMnemonic = accounts.admin.mnemonic;
export const adminAddr = accounts.admin.address;
export const userMnemonic = accounts.user.mnemonic;
export const userAddr = accounts.user.address;
export const guardian1Mnemonic = accounts.guardian_1.mnemonic;
export const guardian1Addr = accounts.guardian_1.address;
export const guardian2Mnemonic = accounts.guardian_2.mnemonic;
export const guardian2Addr = accounts.guardian_2.address;
export const relayer1Mnemonic = accounts.relayer_1.mnemonic;
export const relayer1Addr = accounts.relayer_1.address;
export const relayer2Mnemonic = accounts.relayer_2.mnemonic;
export const relayer2Addr = accounts.relayer_2.address;

export const { proxyCodePath, govecCodePath, factoryCodePath } = getVectisContractPaths(process.env.VECTIS_CW_PATH);
export const { fixMultiSigCodePath, cw20CodePath, daoCodePath, stakingCodePath, voteCodePath, proposalSingleCodePath } =
    getDownloadContractsPath(process.env.DOWNLOADED_CW_PATH);

export const testWalletInitialFunds = coin(5_000_000, coinMinDenom!);
