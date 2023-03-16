import type { UploadResult, Code } from "@cosmjs/cosmwasm-stargate";

export interface ContractsResult {
    host: {
        factoryRes: UploadResult;
        proxyRes: UploadResult;
        daoTunnelRes: UploadResult;
        multisigRes: UploadResult;
        govecRes: UploadResult;
        pluginRegRes: UploadResult;
    };
    remote: {
        remoteTunnel: UploadResult;
        remoteMultisig: UploadResult;
        remoteProxy: UploadResult;
        remoteFactory: UploadResult;
    };
}

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

export interface DaoDaoContracts {
    dao: Code;
    staking: Code;
    vote: Code;
    proposalSingle: Code;
    preProposalSingle: Code;
}
