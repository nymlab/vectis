export type Chains =
    | "juno_testnet"
    | "juno_localnet"
    | "wasm_testnet"
    | "wasm_localnet"
    | "tgrade_localnet"
    | "injective_testnet";
export interface Chain {
    readonly chainId: string;
    readonly chainName: string;
    readonly addressPrefix: string;
    readonly rpcUrl: string;
    readonly httpUrl: string;
    readonly faucetUrl?: string;
    readonly faucetToken?: string;
    readonly feeToken: string;
    readonly stakingToken: string;
    readonly estimatedBlockTime: number;
    readonly estimatedIndexerTime: number;
    readonly gasPrice: number;
    readonly plugins?: string[];
}

export * from "./juno";
export * from "./wasm";
export * from "./tgrade";
export * from "./archway";
export * from "./injective";
