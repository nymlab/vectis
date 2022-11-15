import path from "path";
import * as dotenv from "dotenv";
import * as accounts from "../config/accounts";
import * as chains from "../config/chains";
import type { Chains } from "../config/chains";

dotenv.config({ path: path.join(__dirname, "../.env") });

// Arch
const archSuffix = ""; //process.arch === "arm64" ? "-aarch64" : "";

export const hostChainName = (process.env.HOST_CHAIN || "juno_localnet") as Chains;
export const hostChain = chains[hostChainName as keyof typeof chains];
export const remoteChainName = (process.env.REMOTE_CHAIN || "wasm_localnet") as Chains;
export const remoteChain = chains[remoteChainName as keyof typeof chains];
export const hostAccounts = accounts[hostChainName as keyof typeof accounts];
export const remoteAccounts = accounts[remoteChainName as keyof typeof accounts];

// Contracts Filenames
export const contractsFileNames = {
    vectis_dao_tunnel: "vectis_dao_tunnel.wasm",
    vectis_proxy: `vectis_proxy${archSuffix}.wasm`,
    vectis_factory: `vectis_factory${archSuffix}.wasm`,
    vectis_govec: `vectis_govec${archSuffix}.wasm`,
    vectis_remote_tunnel: "vectis_remote_tunnel.wasm",
    vectis_remote_proxy: "vectis_proxy.wasm",
    vectis_remote_factory: "vectis_remote_factory.wasm",
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
export const cachePath = path.join(__dirname, "../.cache");
export const configPath = path.join(__dirname, "../config");
export const downloadContractPath = path.join(cachePath, "/contracts");
export const uploadReportPath = path.join(cachePath, "uploadInfo.json");
export const deployReportPath = path.join(cachePath, "deployInfo.json");
export const ibcReportPath = path.join(cachePath, "ibcInfo.json");

// Host Contracts
// const vectisContractsPath = process.env.VECTIS_CW_PATH as string;
const vectisContractsPath = path.join(__dirname, "..", "/contracts");
export const daoTunnelCodetPath = path.join(vectisContractsPath, contractsFileNames.vectis_dao_tunnel);
export const proxyCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_proxy);
export const govecCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_govec);
export const factoryCodePath = path.join(vectisContractsPath, contractsFileNames.vectis_factory);

// Remote Contracts
const remoteContractsPath = path.join(__dirname, "..", "/contracts");
export const remoteTunnelCodePath = path.join(remoteContractsPath, contractsFileNames.vectis_remote_tunnel);
export const remoteProxyCodePath = path.join(remoteContractsPath, contractsFileNames.vectis_remote_proxy);
export const remoteFactoryCodePath = path.join(remoteContractsPath, contractsFileNames.vectis_remote_factory);

// CWPlus Contracts
export const fixMultiSigCodePath = path.join(downloadContractPath, contractsFileNames.cw3_mutltisig);
export const cw20CodePath = path.join(downloadContractPath, contractsFileNames.cw20_base);

// DAODAO Contracts
export const daoCodePath = path.join(downloadContractPath, contractsFileNames.cw_dao);
export const stakingCodePath = path.join(downloadContractPath, contractsFileNames.cw20_staking);
export const voteCodePath = path.join(downloadContractPath, contractsFileNames.cw20_voting);
export const proposalSingleCodePath = path.join(downloadContractPath, contractsFileNames.cw_proposal_single);
