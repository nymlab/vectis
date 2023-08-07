import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface VectisContractsUploadResult {
    vectis_factory: UploadResult;
    vectis_proxy: UploadResult;
    cw3Fixed: UploadResult;
    vectis_plugin_registry: UploadResult;
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
