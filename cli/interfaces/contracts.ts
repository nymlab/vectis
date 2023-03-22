import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface DaoContractsUploadResult {
    daoTunnel: UploadResult;
    factory: UploadResult;
    proxy: UploadResult;
    cw3Fixed: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
    govec: UploadResult;
    pluginReg: UploadResult;
    staking: UploadResult;
    dao: UploadResult | Code;
    vote: UploadResult;
    proposalSingle: UploadResult;
    preProposalSingle: UploadResult;
}

export interface RemoteContractsUploadResult {
    remoteTunnel: UploadResult;
    cw3Fixed: UploadResult;
    remoteProxy: UploadResult;
    remoteFactory: UploadResult;
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

// These are all the contract on the Dao-chain
export interface VectisDaoChainContractsAddrs {
    Govec: string;
    DaoTunnel: string;
    ProposalCommittee: string;
    PreProposalModule: string;
    PluginCommittee: string;
    PluginRegistry: string;
    Factory: string;
    Staking: string;
    Dao: string;
    ProposalModule: string;
    PreproposalGroup: string;
    PluginCommitteeGroup: string;
    Vote: string;
}

// These are all the contract on the Remote-chain
export interface VectisRemoteChainContractsAddrs {
    remoteTunnelAddr: string;
    remoteFactoryAddr: string;
}

export interface DaoDaoContracts {
    dao: Code;
}
