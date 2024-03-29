export enum SupportChains {
    "juno_testnet",
    "neutron_testnet",
    "injective_testnet",
    "stargaze_testnet",
    "juno_localnet",
    "osmosis_testnet",
    "aura_testnet",
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
export * from "./neutron";
export * from "./injective";
export * from "./stargaze";
export * from "./osmosis";
export * from "./aura";
