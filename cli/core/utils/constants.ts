import path from "path";
import * as dotenv from "dotenv";
import { coin } from "@cosmjs/stargate";
import networks from "../config/networks.json";
import wallets from "../config/accounts.json";
import { NetworkOptions } from "../interfaces/network";

const archSuffix = process.arch === "arm64" ? "-aarch64" : "";
const cwPlusReleaseVer = "v0.13.4";

const envPath = `${__dirname}/../../../.env`;
dotenv.config({ path: envPath });

// Network
const network = networks[process.env.NETWORK as NetworkOptions];
export const chainId = network.chainId;
export const addrPrefix = network.addressPrefix;
export const rpcEndPoint = network.rpcUrl;
export const gasPrice = network.gasPrice + network.feeToken;
export const coinDenom = network.feeToken;
export const coinMinDenom = network.feeToken;

// Accounts
const accounts = wallets[network.addressPrefix as "juno" | "wasm"];
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

// Paths
export const cachePath = path.join(__dirname, "..", "..", ".cache");
export const downloadContractPath = path.join(cachePath, "/contracts");
export const uploadReportPath = path.join(cachePath, "uploadInfo.json");

export const vectisContractsPath = process.env.VECTIS_CW_PATH as string;
export const proxyCodePath = path.join(vectisContractsPath, `vectis_proxy${archSuffix}.wasm`);
export const govecCodePath = path.join(vectisContractsPath, `vectis_govec${archSuffix}.wasm`);
export const factoryCodePath = path.join(vectisContractsPath, `vectis_factory${archSuffix}.wasm`);

export const wasmContractsPath = process.env.DOWNLOADED_CW_PATH as string;
export const fixMultiSigCodePath = path.join(downloadContractPath, "cw3_fixed_multisig.wasm");
export const cw20CodePath = path.join(downloadContractPath, "cw20_base.wasm");
export const daoCodePath = path.join(wasmContractsPath, "cw_core.wasm");
export const stakingCodePath = path.join(wasmContractsPath, "stake_cw20.wasm");
export const voteCodePath = path.join(wasmContractsPath, "cw20_staked_balance_voting.wasm");
export const proposalSingleCodePath = path.join(wasmContractsPath, "cw_proposal_single.wasm");

// Contracts Links
export const cw20BaseDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw20_base.wasm`;
export const cw3FixedMulDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_fixed_multisig.wasm`;

export const testWalletInitialFunds = coin(5_000_000, coinMinDenom!);
