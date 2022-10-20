/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import {
    InstantiateMsg,
    ExecuteMsg,
    CosmosMsgForEmpty,
    BankMsg,
    Uint128,
    Binary,
    IbcMsg,
    Timestamp,
    Uint64,
    WasmMsg,
    GovMsg,
    VoteOption,
    Coin,
    Empty,
    IbcTimeout,
    IbcTimeoutBlock,
    QueryMsg,
    Nullable_String,
    Nullable_Addr,
    Addr,
} from "./RemoteTunnel.types";
export interface RemoteTunnelReadOnlyInterface {
    contractAddress: string;
    factory: () => Promise<NullableAddr>;
    channel: () => Promise<NullableString>;
}
export class RemoteTunnelQueryClient implements RemoteTunnelReadOnlyInterface {
    client: CosmWasmClient;
    contractAddress: string;

    constructor(client: CosmWasmClient, contractAddress: string) {
        this.client = client;
        this.contractAddress = contractAddress;
        this.factory = this.factory.bind(this);
        this.channel = this.channel.bind(this);
    }

    factory = async (): Promise<NullableAddr> => {
        return this.client.queryContractSmart(this.contractAddress, {
            factory: {},
        });
    };
    channel = async (): Promise<NullableString> => {
        return this.client.queryContractSmart(this.contractAddress, {
            channel: {},
        });
    };
}
export interface RemoteTunnelInterface extends RemoteTunnelReadOnlyInterface {
    contractAddress: string;
    sender: string;
    mintGovec: (
        {
            walletAddr,
        }: {
            walletAddr: string;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    dispatch: (
        {
            jobId,
            msgs,
        }: {
            jobId?: string;
            msgs: CosmosMsgForEmpty[];
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
}
export class RemoteTunnelClient extends RemoteTunnelQueryClient implements RemoteTunnelInterface {
    client: SigningCosmWasmClient;
    sender: string;
    contractAddress: string;

    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, contractAddress);
        this.client = client;
        this.sender = sender;
        this.contractAddress = contractAddress;
        this.mintGovec = this.mintGovec.bind(this);
        this.dispatch = this.dispatch.bind(this);
    }

    mintGovec = async (
        {
            walletAddr,
        }: {
            walletAddr: string;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                mint_govec: {
                    wallet_addr: walletAddr,
                },
            },
            fee,
            memo,
            funds
        );
    };
    dispatch = async (
        {
            jobId,
            msgs,
        }: {
            jobId?: string;
            msgs: CosmosMsgForEmpty[];
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                dispatch: {
                    job_id: jobId,
                    msgs,
                },
            },
            fee,
            memo,
            funds
        );
    };
}
