import path from "path";
import * as dotenv from "dotenv";
import * as accounts from "../config/accounts";
import * as chains from "../config/chains";
import type { Chain } from "../config/chains";

dotenv.config({ path: path.join(__dirname, "../../.env") });

// This is manual translate onchain DaoActors to string
export enum VectisActors {
    PluginCommittee = "PluginCommittee",
    PluginRegistry = "PluginRegistry",
    Factory = "Factory",
}

// Contracts Filenames
export const contractsFileNames = {
    vectis_proxy: `vectis_proxy.wasm`,
    vectis_factory: `vectis_factory.wasm`,
    vectis_plugin_registry: "vectis_plugin_registry.wasm",
    vectis_cronkitty: `cronkitty.wasm`,
    cw3_mutltisig: "cw3_fixed_multisig.wasm",
    cw3_flex_mutltisig: "cw3_flex_multisig.wasm",
    cw4_group: "cw4_group.wasm",
};

// Contracts Versioning
// This is branched off v1.0.1
const cwPlusReleaseVer = "vectis-beta-v1";

// Contracts Links CWPlus contracts
// Diff is minor, we just add a state in the cw3-flex and exec msg to change it
export const cw3FixedMulDownloadLink = `https://github.com/nymlab/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_fixed_multisig.wasm`;
export const cw3FlexMulDownloadLink = `https://github.com/nymlab/cw-plus/releases/download/${cwPlusReleaseVer}/cw3_flex_multisig.wasm`;
export const cw4GroupDownloadLink = `https://github.com/nymlab/cw-plus/releases/download/${cwPlusReleaseVer}/cw4_group.wasm`;

// Schema links
// We only need these because we want to generate clients, we are not generating a cw3-fixed-multisig client
export const cw3flexSchemaLink = `https://github.com/nymlab/cw-plus/releases/download/${cwPlusReleaseVer}/cw3-flex-multisig.json`;
export const cw4GroupSchemaLink = `https://github.com/nymlab/cw-plus/releases/download/${cwPlusReleaseVer}/cw4-group.json`;

// Paths
export const cachePath = path.join(__dirname, "../../.cache");
export const deployPath = path.join(__dirname, "../../deploy");
export const configPath = path.join(__dirname, "../config");
export const downloadContractPath = path.join(cachePath, "/contracts");
export const downloadSchemaPath = path.join(cachePath, "/schemas");
const vectisContractsPath = path.join(__dirname, "../../../artifacts");

// Deploy output paths
export const hubUploadReportPath = path.join(
    process.env.HOST_CHAIN == undefined || process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    `${process.env.HOST_CHAIN}-uploadInfo.json`
);
export const hubDeployReportPath = path.join(
    process.env.HOST_CHAIN == undefined || process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    `${process.env.HOST_CHAIN}-deployInfo.json`
);
export const ibcReportPath = path.join(
    process.env.HOST_CHAIN == undefined || process.env.HOST_CHAIN == "juno_localnet" ? cachePath : deployPath,
    "ibcInfo.json"
);

export const coreCodePaths: { [index: string]: string } = {
    proxyCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_proxy),
    pluginRegCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_plugin_registry),
    factoryCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_factory),

    // CWPlus Contracts
    cw3FixedCodePath: path.join(downloadContractPath, contractsFileNames.cw3_mutltisig),
    cw3FlexCodePath: path.join(downloadContractPath, contractsFileNames.cw3_flex_mutltisig),
    cw4GroupCodePath: path.join(downloadContractPath, contractsFileNames.cw4_group),
};

export const pluginCodePaths: { [index: string]: string } = {
    cronkittyCodePath: path.join(vectisContractsPath, contractsFileNames.vectis_cronkitty),
};
