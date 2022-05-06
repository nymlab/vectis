/**
 * This file was automatically generated by cosmwasm-typescript-gen.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the cosmwasm-typescript-gen generate command to regenerate this file.
 */

import { CosmWasmClient, ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
export type CodeIdResponse = number;
export type CodeIdType = "Proxy" | "Multisig" | "Govec" | "Staking";
export type Uint128 = string;
export type Binary = string;
export interface CreateWalletMsg {
    guardians: Guardians;
    proxy_initial_funds: Coin[];
    relayers: string[];
    user_pubkey: Binary;
    [k: string]: unknown;
}
export interface Guardians {
    addresses: string[];
    guardians_multisig?: MultiSig | null;
    [k: string]: unknown;
}
export interface MultiSig {
    multisig_initial_funds: Coin[];
    threshold_absolute_count: number;
    [k: string]: unknown;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export interface Cw20Coin {
    address: string;
    amount: Uint128;
    [k: string]: unknown;
}
export interface FeeResponse {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export interface InstantiateMsg {
    addr_prefix: string;
    govec_code_id: number;
    proxy_code_id: number;
    proxy_multisig_code_id: number;
    staking_code_id: number;
    wallet_fee: Coin;
    [k: string]: unknown;
}
export type ProxyMigrationTxMsg =
    | {
          RelayTx: RelayTransaction;
      }
    | {
          DirectMigrationMsg: Binary;
      };
export interface RelayTransaction {
    message: Binary;
    nonce: number;
    signature: Binary;
    user_pubkey: Binary;
    [k: string]: unknown;
}
export type Duration =
    | {
          height: number;
      }
    | {
          time: number;
      };
export interface StakingOptions {
    code_id: number;
    duration?: Duration | null;
    [k: string]: unknown;
}
export type WalletAddr =
    | {
          Canonical: Binary;
      }
    | {
          Addr: Addr;
      };
export type Addr = string;
export interface WalletInfo {
    code_id: number;
    guardians: Addr[];
    is_frozen: boolean;
    multisig_address?: Addr | null;
    multisig_code_id: number;
    nonce: number;
    relayers: Addr[];
    user_addr: Addr;
    version: ContractVersion;
    [k: string]: unknown;
}
export interface ContractVersion {
    contract: string;
    version: string;
    [k: string]: unknown;
}
export interface WalletQueryPrefix {
    user_addr: string;
    wallet_addr: string;
    [k: string]: unknown;
}
export interface WalletsOfResponse {
    wallets: Addr[];
    [k: string]: unknown;
}
export interface WalletsResponse {
    wallets: Addr[];
    [k: string]: unknown;
}
export interface FactoryReadOnlyInterface {
    contractAddress: string;
    wallets: ({ limit, startAfter }: { limit?: number; startAfter?: WalletQueryPrefix }) => Promise<WalletsResponse>;
    walletsOf: ({
        limit,
        startAfter,
        user,
    }: {
        limit?: number;
        startAfter?: string;
        user: string;
    }) => Promise<WalletsOfResponse>;
    codeId: ({ ty }: { ty: CodeIdType }) => Promise<CodeIdResponse>;
    fee: () => Promise<FeeResponse>;
}
export class FactoryQueryClient implements FactoryReadOnlyInterface {
    client: CosmWasmClient;
    contractAddress: string;

    constructor(client: CosmWasmClient, contractAddress: string) {
        this.client = client;
        this.contractAddress = contractAddress;
        this.wallets = this.wallets.bind(this);
        this.walletsOf = this.walletsOf.bind(this);
        this.codeId = this.codeId.bind(this);
        this.fee = this.fee.bind(this);
    }

    wallets = async ({
        limit,
        startAfter,
    }: {
        limit?: number;
        startAfter?: WalletQueryPrefix;
    }): Promise<WalletsResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            wallets: {
                limit,
                start_after: startAfter,
            },
        });
    };
    walletsOf = async ({
        limit,
        startAfter,
        user,
    }: {
        limit?: number;
        startAfter?: string;
        user: string;
    }): Promise<WalletsOfResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            wallets_of: {
                limit,
                start_after: startAfter,
                user,
            },
        });
    };
    codeId = async ({ ty }: { ty: CodeIdType }): Promise<CodeIdResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            code_id: {
                ty,
            },
        });
    };
    fee = async (): Promise<FeeResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            fee: {},
        });
    };
}
export interface FactoryInterface extends FactoryReadOnlyInterface {
    contractAddress: string;
    sender: string;
    createWallet: (
        {
            createWalletMsg,
        }: {
            createWalletMsg: CreateWalletMsg;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: readonly Coin[]
    ) => Promise<ExecuteResult>;
    migrateWallet: (
        {
            migrationMsg,
            walletAddress,
        }: {
            migrationMsg: ProxyMigrationTxMsg;
            walletAddress: WalletAddr;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: readonly Coin[]
    ) => Promise<ExecuteResult>;
    updateCodeId: (
        {
            newCodeId,
            ty,
        }: {
            newCodeId: number;
            ty: CodeIdType;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: readonly Coin[]
    ) => Promise<ExecuteResult>;
    updateWalletFee: (
        {
            newFee,
        }: {
            newFee: Coin;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: readonly Coin[]
    ) => Promise<ExecuteResult>;
    createGovernance: (
        {
            initialBalances,
            stakingOptions,
        }: {
            initialBalances: Cw20Coin[];
            stakingOptions?: StakingOptions;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: readonly Coin[]
    ) => Promise<ExecuteResult>;
}
export class FactoryClient extends FactoryQueryClient implements FactoryInterface {
    override client: SigningCosmWasmClient;
    sender: string;
    override contractAddress: string;

    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, contractAddress);
        this.client = client;
        this.sender = sender;
        this.contractAddress = contractAddress;
        this.createWallet = this.createWallet.bind(this);
        this.migrateWallet = this.migrateWallet.bind(this);
        this.updateCodeId = this.updateCodeId.bind(this);
        this.updateWalletFee = this.updateWalletFee.bind(this);
        this.createGovernance = this.createGovernance.bind(this);
    }

    createWallet = async (
        {
            createWalletMsg,
        }: {
            createWalletMsg: CreateWalletMsg;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: readonly Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                create_wallet: {
                    create_wallet_msg: createWalletMsg,
                },
            },
            fee,
            memo,
            funds
        );
    };
    migrateWallet = async (
        {
            migrationMsg,
            walletAddress,
        }: {
            migrationMsg: ProxyMigrationTxMsg;
            walletAddress: WalletAddr;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: readonly Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                migrate_wallet: {
                    migration_msg: migrationMsg,
                    wallet_address: walletAddress,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateCodeId = async (
        {
            newCodeId,
            ty,
        }: {
            newCodeId: number;
            ty: CodeIdType;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: readonly Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_code_id: {
                    new_code_id: newCodeId,
                    ty,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateWalletFee = async (
        {
            newFee,
        }: {
            newFee: Coin;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: readonly Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_wallet_fee: {
                    new_fee: newFee,
                },
            },
            fee,
            memo,
            funds
        );
    };
    createGovernance = async (
        {
            initialBalances,
            stakingOptions,
        }: {
            initialBalances: Cw20Coin[];
            stakingOptions?: StakingOptions;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: readonly Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                create_governance: {
                    initial_balances: initialBalances,
                    staking_options: stakingOptions,
                },
            },
            fee,
            memo,
            funds
        );
    };
}
