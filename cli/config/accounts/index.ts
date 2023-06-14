export type Accounts =
    | "admin"
    | "user"
    | "guardian_1"
    | "guardian_2"
    | "relayer_1"
    | "relayer_2"
    | "committee1"
    | "committee2";
export interface Account {
    address: string;
    mnemonic: string;
}

export * from "./juno";
export * from "./wasm";
export * from "./stargaze";
export * from "./injective";
export * from "./archway";
export * from "./neutron";
