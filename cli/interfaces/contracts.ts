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

// This is used for testing only
export interface VectisDaoContractsAddrs {
    remoteTunnelAddr: string;
    remoteFactoryAddr: string;
    daoTunnelAddr: string;
    daoAddr: string;
    govecAddr: string;
    factoryAddr: string;
    stakingAddr: string;
    proposalAddr: string;
    preproposalAddr: string;
    preProposalMultiSigAddr: string;
    preproposalGroupAddr: string;
    pluginRegistryAddr: string;
    techCommitteeMultiSigAddr: string;
    techCommitteeGroupAddr: string;
    voteAddr: string;
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
