export interface VectisDaoContractsAddrs {
    daoAddr: string;
    govecAddr: string;
    factoryAddr: string;
    stakingAddr: string;
    proposalAddr: string;
    voteAddr: string;
}

export interface DaoDaoContracts {
    [index: string]: any;
    dao: any;
    staking: any;
    vote: any;
    proposalSingle: any;
}
