export type Accounts =
    | "admin"
    | "user"
    | "guardian_1"
    | "guardian_2"
    | "relayer_1"
    | "relayer_2"
    | "token_holder"
    | "committee1"
    | "committee2";
export interface Account {
    address: string;
    mnemonic: string;
}

export * from "./juno";
export * from "./wasm";
