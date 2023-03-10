/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import {
    Uint128,
    LogoInfo,
    Addr,
    InstantiateMsg,
    Cw20Coin,
    MarketingInfoResponse,
    ExecuteMsg,
    Binary,
    Logo,
    EmbeddedLogo,
    QueryMsg,
    AllAccountsResponse,
    BalanceResponse,
    DownloadLogoResponse,
    NullableBalanceResponse,
    MintResponse,
    String,
    TokenInfoResponse,
} from "./Govec.types";
export interface GovecReadOnlyInterface {
    contractAddress: string;
    balance: ({ address }: { address: string }) => Promise<BalanceResponse>;
    joined: ({ address }: { address: string }) => Promise<NullableBalanceResponse>;
    tokenInfo: () => Promise<TokenInfoResponse>;
    mintAmount: () => Promise<Uint128>;
    minters: () => Promise<MintResponse>;
    dao: () => Promise<Addr>;
    allAccounts: ({ limit, startAfter }: { limit?: number; startAfter?: string }) => Promise<AllAccountsResponse>;
    marketingInfo: () => Promise<MarketingInfoResponse>;
    downloadLogo: () => Promise<DownloadLogoResponse>;
    tokenContract: () => Promise<String>;
}
export class GovecQueryClient implements GovecReadOnlyInterface {
    client: CosmWasmClient;
    contractAddress: string;

    constructor(client: CosmWasmClient, contractAddress: string) {
        this.client = client;
        this.contractAddress = contractAddress;
        this.balance = this.balance.bind(this);
        this.joined = this.joined.bind(this);
        this.tokenInfo = this.tokenInfo.bind(this);
        this.mintAmount = this.mintAmount.bind(this);
        this.minters = this.minters.bind(this);
        this.dao = this.dao.bind(this);
        this.allAccounts = this.allAccounts.bind(this);
        this.marketingInfo = this.marketingInfo.bind(this);
        this.downloadLogo = this.downloadLogo.bind(this);
        this.tokenContract = this.tokenContract.bind(this);
    }

    balance = async ({ address }: { address: string }): Promise<BalanceResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            balance: {
                address,
            },
        });
    };
    joined = async ({ address }: { address: string }): Promise<NullableBalanceResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            joined: {
                address,
            },
        });
    };
    tokenInfo = async (): Promise<TokenInfoResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            token_info: {},
        });
    };
    mintAmount = async (): Promise<Uint128> => {
        return this.client.queryContractSmart(this.contractAddress, {
            mint_amount: {},
        });
    };
    minters = async (): Promise<MintResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            minters: {},
        });
    };
    dao = async (): Promise<Addr> => {
        return this.client.queryContractSmart(this.contractAddress, {
            dao: {},
        });
    };
    allAccounts = async ({
        limit,
        startAfter,
    }: {
        limit?: number;
        startAfter?: string;
    }): Promise<AllAccountsResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            all_accounts: {
                limit,
                start_after: startAfter,
            },
        });
    };
    marketingInfo = async (): Promise<MarketingInfoResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            marketing_info: {},
        });
    };
    downloadLogo = async (): Promise<DownloadLogoResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            download_logo: {},
        });
    };
    tokenContract = async (): Promise<String> => {
        return this.client.queryContractSmart(this.contractAddress, {
            token_contract: {},
        });
    };
}
export interface GovecInterface extends GovecReadOnlyInterface {
    contractAddress: string;
    sender: string;
    transfer: (
        {
            amount,
            recipient,
            relayedFrom,
        }: {
            amount: Uint128;
            recipient: string;
            relayedFrom?: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    transferFrom: (
        {
            amount,
            owner,
            recipient,
        }: {
            amount: Uint128;
            owner: string;
            recipient: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    burn: (
        {
            amount,
        }: {
            amount: Uint128;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    exit: (
        {
            relayedFrom,
        }: {
            relayedFrom?: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    send: (
        {
            amount,
            contract,
            msg,
            relayedFrom,
        }: {
            amount: Uint128;
            contract: string;
            msg: Binary;
            relayedFrom?: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    mint: (
        {
            newWallet,
        }: {
            newWallet: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    updateMintCap: (
        {
            newMintCap,
        }: {
            newMintCap?: Uint128;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    updateMintAmount: (
        {
            newAmount,
        }: {
            newAmount: Uint128;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    updateDaoAddr: (
        {
            newAddr,
        }: {
            newAddr: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    updateMarketing: (
        {
            description,
            marketing,
            project,
        }: {
            description?: string;
            marketing?: string;
            project?: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    uploadLogo: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class GovecClient extends GovecQueryClient implements GovecInterface {
    override client: SigningCosmWasmClient;
    sender: string;
    override contractAddress: string;

    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, contractAddress);
        this.client = client;
        this.sender = sender;
        this.contractAddress = contractAddress;
        this.transfer = this.transfer.bind(this);
        this.transferFrom = this.transferFrom.bind(this);
        this.burn = this.burn.bind(this);
        this.exit = this.exit.bind(this);
        this.send = this.send.bind(this);
        this.mint = this.mint.bind(this);
        this.updateMintCap = this.updateMintCap.bind(this);
        this.updateMintAmount = this.updateMintAmount.bind(this);
        this.updateDaoAddr = this.updateDaoAddr.bind(this);
        this.updateMarketing = this.updateMarketing.bind(this);
        this.uploadLogo = this.uploadLogo.bind(this);
    }

    transfer = async (
        {
            amount,
            recipient,
            relayedFrom,
        }: {
            amount: Uint128;
            recipient: string;
            relayedFrom?: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                transfer: {
                    amount,
                    recipient,
                    relayed_from: relayedFrom,
                },
            },
            fee,
            memo,
            funds
        );
    };
    transferFrom = async (
        {
            amount,
            owner,
            recipient,
        }: {
            amount: Uint128;
            owner: string;
            recipient: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                transfer_from: {
                    amount,
                    owner,
                    recipient,
                },
            },
            fee,
            memo,
            funds
        );
    };
    burn = async (
        {
            amount,
        }: {
            amount: Uint128;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                burn: {
                    amount,
                },
            },
            fee,
            memo,
            funds
        );
    };
    exit = async (
        {
            relayedFrom,
        }: {
            relayedFrom?: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                exit: {
                    relayed_from: relayedFrom,
                },
            },
            fee,
            memo,
            funds
        );
    };
    send = async (
        {
            amount,
            contract,
            msg,
            relayedFrom,
        }: {
            amount: Uint128;
            contract: string;
            msg: Binary;
            relayedFrom?: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                send: {
                    amount,
                    contract,
                    msg,
                    relayed_from: relayedFrom,
                },
            },
            fee,
            memo,
            funds
        );
    };
    mint = async (
        {
            newWallet,
        }: {
            newWallet: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                mint: {
                    new_wallet: newWallet,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateMintCap = async (
        {
            newMintCap,
        }: {
            newMintCap?: Uint128;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_mint_cap: {
                    new_mint_cap: newMintCap,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateMintAmount = async (
        {
            newAmount,
        }: {
            newAmount: Uint128;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_mint_amount: {
                    new_amount: newAmount,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateDaoAddr = async (
        {
            newAddr,
        }: {
            newAddr: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_dao_addr: {
                    new_addr: newAddr,
                },
            },
            fee,
            memo,
            funds
        );
    };
    updateMarketing = async (
        {
            description,
            marketing,
            project,
        }: {
            description?: string;
            marketing?: string;
            project?: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                update_marketing: {
                    description,
                    marketing,
                    project,
                },
            },
            fee,
            memo,
            funds
        );
    };
    uploadLogo = async (
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                upload_logo: {},
            },
            fee,
            memo,
            funds
        );
    };
}
