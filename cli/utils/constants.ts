import path from "path";
import * as dotenv from "dotenv";
import * as accounts from "../config/accounts";
import * as chains from "../config/chains";
import type { Chains } from "../config/chains";

dotenv.config({ path: path.join(__dirname, "../.env") });

export const hostChainName = (process.env.HOST_CHAIN || "juno_localnet") as Chains;
export const hostChain = chains[hostChainName as keyof typeof chains];
export const remoteChainName = (process.env.REMOTE_CHAIN || "wasm_localnet") as Chains;
export const remoteChain = chains[remoteChainName as keyof typeof chains];
export const hostAccounts = accounts[hostChainName as keyof typeof accounts];
export const remoteAccounts = accounts[remoteChainName as keyof typeof accounts];

// This is manual translate onchain DaoActors to string
export enum DaoActors {
    Govec = "Govec",
    DaoTunnel = "DaoTunnel",
    ProposalCommittee = "ProposalCommittee",
    PreProposalModule = "PreProposalModule",
    PluginCommittee = "PluginCommittee",
    PluginRegistry = "PluginRegistry",
    Factory = "Factory",
    TreasuryCommittee = "TreasuryCommittee",
    Staking = "Staking",
}

// Contracts Filenames
export const contractsFileNames = {
    vectis_dao_tunnel: "vectis_dao_tunnel.wasm",
    vectis_proxy: `vectis_proxy.wasm`,
    vectis_factory: `vectis_factory.wasm`,
    vectis_govec: `vectis_govec.wasm`,
    vectis_remote_tunnel: "vectis_remote_tunnel.wasm",
    vectis_remote_proxy: "vectis_proxy.wasm",
    vectis_plugin_registry: "vectis_plugin_registry.wasm",
    vectis_remote_factory: "vectis_remote_factory.wasm",
    cw3_mutltisig: "cw3_fixed_multisig.wasm",
    cw3_flex_mutltisig: "cw3_flex_multisig.wasm",
    cw4_group: "cw4_group.wasm",
    cw20_base: "cw20_base.wasm",
    cw_dao: "dao_core.wasm",
    cw20_staking: "cw20_stake.wasm",
    cw20_voting: "dao_voting_cw20_staked.wasm",
    cw_proposal_single: "dao_proposal_single.wasm",
    cw_pre_proposal_approval_single: "dao_pre_propose_approval_single.wasm",
};

// Contracts Versioning
const cwPlusReleaseVer = "v0.16.0";
const daodaoReleaseVer = "v2.0.1-vectis-1.0";
// Contracts Links CWPlus
export const cw20BaseDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw20_base.wasm`;
export const cw3FixedMulDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_fixed_multisig.wasm`;
export const cw3FlexMulDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_flex_multisig.wasm`;
export const cw4GroupDownloadLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw4_group.wasm`;

// Schema links
export const cw3flexSchemaLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw3-flex-multisig.json`;
export const cw4GroupSchemaLink = `https://github.com/CosmWasm/cw-plus/releases/download/${cwPlusReleaseVer}/cw4-group.json`;

// Contracts Links dao-contract (Vectis)
export const cwDaoDownloadLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao_core.wasm`;
export const cw20StakingDownloadLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/cw20_stake.wasm`;
export const cw20VotingDownloadLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao_voting_cw20_staked.wasm`;
export const cw20ProposalSingleDownloadLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao_proposal_single.wasm`;
export const cwPreProposalSingleDownloadLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao_pre_propose_approval_single.wasm`;

// Schema Links dao-contract types (Vectis)
export const proposalSingleSchemaLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao-proposal-single.json`;
export const prePropSingleApprovalSchemaLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/dao-pre-propose-approval-single.json`;
export const cw20StakeSchemaLink = `https://github.com/nymlab/dao-contracts/releases/download/${daodaoReleaseVer}/cw20-stake.json`;

// Paths
export const cachePath = path.join(__dirname, "../.cache");
export const deployPath = path.join(__dirname, "../deploy");
export const configPath = path.join(__dirname, "../config");
export const contractPath = path.join(__dirname, "../contracts");
export const downloadContractPath = path.join(cachePath, "/contracts");
export const downloadSchemaPath = path.join(cachePath, "/schemas");

// Deploy output paths
export const daoUploadReportPath = path.join(
    process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    `${process.env.HOST_CHAIN}-uploadInfo.json`
);
export const remoteUploadReportPath = path.join(
    process.env.REMOTE_CHAIN == "wasm_localnet" ? cachePath : deployPath,
    `${process.env.REMOTE_CHAIN}-uploadInfo.json`
);
export const daoDeployReportPath = path.join(
    process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    `${process.env.HOST_CHAIN}-deployInfo.json`
);
export const remoteDeployReportPath = path.join(
    process.env.REMOTE_CHAIN == "wasm_localnet" ? cachePath : deployPath,
    `${process.env.REMOTE_CHAIN}-deployInfo.json`
);
export const ibcReportPath = path.join(
    process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    "ibcInfo.json"
);

// Host Contracts
const vectisContractsPath = path.join(__dirname, "../../artifacts");
const remoteContractsPath = path.join(__dirname, "../../artifacts");

// Wasm file paths
export const codePaths: { [index: string]: string } = {
    daoTunnelCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_dao_tunnel),
    proxyCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_proxy),
    govecCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_govec),
    pluginRegCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_plugin_registry),
    factoryCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_factory),

    // Remote Contracts
    remoteTunnelCodePath: path.join(remoteContractsPath, contractsFileNames.vectis_remote_tunnel),
    remoteProxyCodePath: path.join(remoteContractsPath, contractsFileNames.vectis_remote_proxy),
    remoteFactoryCodePath: path.join(remoteContractsPath, contractsFileNames.vectis_remote_factory),

    // CWPlus Contracts
    cw3FixedCodePath: path.join(downloadContractPath, contractsFileNames.cw3_mutltisig),
    cw20CodePath: path.join(downloadContractPath, contractsFileNames.cw20_base),
    cw3FlexCodePath: path.join(downloadContractPath, contractsFileNames.cw3_flex_mutltisig),
    cw4GroupCodePath: path.join(downloadContractPath, contractsFileNames.cw4_group),

    // DAODAO Contracts
    daoCodePath: path.join(downloadContractPath, contractsFileNames.cw_dao),
    stakingCodePath: path.join(downloadContractPath, contractsFileNames.cw20_staking),
    voteCodePath: path.join(downloadContractPath, contractsFileNames.cw20_voting),
    proposalSingleCodePath: path.join(downloadContractPath, contractsFileNames.cw_proposal_single),
    preProposalSingleCodePath: path.join(downloadContractPath, contractsFileNames.cw_pre_proposal_approval_single),
};
