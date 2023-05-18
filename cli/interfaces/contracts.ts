import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface HubContractsUploadResult {
    factory: UploadResult;
    proxy: UploadResult;
    cw3Fixed: UploadResult;
    pluginReg: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
    plugins: { [key: string]: UploadResult };
}

// These are all the contract on the Hub Chain
export interface VectisHubChainContractsAddrs {
    PluginCommittee: string;
    PluginCommitteeGroup: string;
    VectisCommittee: string;
    VectisCommitteeGroup: string;
    Factory: string;
    PluginRegistry: string;
}
