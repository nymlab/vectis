import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface VectisContractsUploadResult {
    factory: UploadResult;
    proxy: UploadResult;
    cw3Fixed: UploadResult;
    pluginReg: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
}

// These are all the contract
export interface VectisContractsAddrs {
    PluginCommittee: string;
    PluginCommitteeGroup: string;
    VectisCommittee: string;
    VectisCommitteeGroup: string;
    Factory: string;
    PluginRegistry: string;
}
