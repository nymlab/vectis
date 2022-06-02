export type NetworkOptions = "juno_testnet" | "juno_local" | "wasmd_testnet" | "wasmd_local";

export interface Network {
    readonly chainId: string;
    readonly chainName: string;
    readonly addressPrefix: string;
    readonly rpcUrl: string;
    readonly httpUrl: string;
    readonly faucetUrl?: string;
    readonly faucetToken?: string;
    readonly feeToken: string;
    readonly stakingToken: string;
    readonly coinMap: { [key: string]: { denom: string; fractionalDigits: number } };
    readonly gasPrice: number;
}
