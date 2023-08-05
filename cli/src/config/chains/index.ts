export enum SupportChains {
    "juno_testnet",
    "neutron_testnet",
    "injective_testnet",
    "archway_testnet",
    "stargaze_testnet",
    "juno_localnet",
<<<<<<< HEAD
    "osmosis_testnet",
=======
    "aura_testnet",
>>>>>>> 5729380 (wip: cli)
}

export type Chains = keyof typeof SupportChains;

export interface Chain {
    readonly chainId: string;
    readonly chainName: string;
    readonly addressPrefix: string;
    readonly rpcUrl: string;
    readonly httpUrl: string;
    readonly exponent: number;
    readonly faucetUrl?: string;
    readonly faucetToken?: string;
    readonly feeToken: string;
    readonly stakingToken: string;
    readonly estimatedBlockTime: number;
    readonly estimatedIndexerTime: number;
    readonly gasPrice: number;
    readonly coinType: number;
}

export * from "./juno";
export * from "./archway";
export * from "./neutron";
export * from "./injective";
export * from "./stargaze";
<<<<<<< HEAD
export * from "./osmosis";
=======
export * from "./aura";
>>>>>>> 5729380 (wip: cli)
