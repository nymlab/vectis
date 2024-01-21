export enum TestAccountNames {
    "admin",
    "user",
    "guardian_1",
    "guardian_2",
    "relayer_1",
    "relayer_2",
    "committee1",
    "committee2",
    "walletCreator"
}

export type Accounts = keyof typeof TestAccountNames;

export interface Account {
    address: string;
    mnemonic: string;
}

export * from "./networks";
