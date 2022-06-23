import path from "path";
import * as dotenv from "dotenv";
import { coin } from "@cosmjs/stargate";
import networks from "../config/networks.json";
import wallets from "../config/accounts.json";
import { NetworkOptions } from "../interfaces/network";

const envPath = `${__dirname}/../../../.env`;
dotenv.config({ path: envPath });

// Arch
const archSuffix = process.arch === "arm64" ? "-aarch64" : "";

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

// Contracts Filenames
export const contractsFileNames = {
    vectis_proxy: `vectis_proxy${archSuffix}.wasm`,
    vectis_factory: `vectis_factory${archSuffix}.wasm`,
    vectis_govec: `vectis_govec${archSuffix}.wasm`,
    cw3_mutltisig: "cw3_fixed_multisig.wasm",
    cw20_base: "cw20_base.wasm",
    cw_dao: "cw_core.wasm",
    cw20_staking: "cw20_stake.wasm",
    cw20_voting: "cw20_staked_balance_voting.wasm",
    cw_proposal_single: "cw_proposal_single.wasm",
};

// Contracts Versioning
const cwPlusReleaseVer = "v0.13.4";
const daodaoReleaseVer = "v1.0.0";

// Contracts Links CWPlus
export const cw20BaseDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw20_base.wasm`;
export const cw3FixedMulDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_fixed_multisig.wasm`;

// Contracts Links DAODAO
export const cwDaoDownloadLink = `https://github.com/DA0-DA0/dao-contracts/releases/download/${daodaoReleaseVer}/cw_core.wasm`;
export const cw20StakingDownloadLink = `https://github.com/DA0-DA0/dao-contracts/releases/download/${daodaoReleaseVer}/cw20_stake.wasm`;
export const cw20VotingDownloadLink = `https://github.com/DA0-DA0/dao-contracts/releases/download/${daodaoReleaseVer}/cw20_staked_balance_voting.wasm`;
export const cw20ProposalSingleDownloadLink = `https://github.com/DA0-DA0/dao-contracts/releases/download/${daodaoReleaseVer}/cw_proposal_single.wasm`;

// Paths
export const cachePath = path.join(__dirname, "..", "..", ".cache");
export const downloadContractPath = path.join(cachePath, "/contracts");
export const uploadReportPath = path.join(cachePath, "uploadInfo.json");

export const vectisContractsPath = process.env.VECTIS_CW_PATH as string;
export const proxyCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_proxy);
export const govecCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_govec);
export const factoryCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_factory);

export const fixMultiSigCodePath = path.join(downloadContractPath, contractsFileNames.cw3_mutltisig);
export const cw20CodePath = path.join(downloadContractPath, contractsFileNames.cw20_base);
export const daoCodePath = path.join(downloadContractPath, contractsFileNames.cw_dao);
export const stakingCodePath = path.join(downloadContractPath, contractsFileNames.cw20_staking);
export const voteCodePath = path.join(downloadContractPath, contractsFileNames.cw20_voting);
export const proposalSingleCodePath = path.join(downloadContractPath, contractsFileNames.cw_proposal_single);

export const testWalletInitialFunds = coin(5_000_000, coinMinDenom!);
