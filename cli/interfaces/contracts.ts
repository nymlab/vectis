import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface HubContractsUploadResult {
    factory: UploadResult;
    proxy: UploadResult;
    cw3Fixed: UploadResult;
    pluginReg: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
}

export interface RemoteContractsUploadResult {
    factory: UploadResult;
    proxy: UploadResult;
    cw3Fixed: UploadResult;
    pluginReg: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
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

// These are all the contract on the Remote-chain
export interface VectisRemoteChainContractsAddrs {
    remoteTunnelAddr: string;
    remoteFactoryAddr: string;
}
