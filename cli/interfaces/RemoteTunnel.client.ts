/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.22.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import {
    CanonicalAddr,
    Binary,
    InstantiateMsg,
    ChainConfig,
    DaoConfig,
    IbcTransferChannels,
    ExecuteMsg,
    RemoteTunnelPacketMsg,
    GovecExecuteMsg,
    Uint128,
    UpdateAddrReq,
    Logo,
    EmbeddedLogo,
    ExecuteMsg1,
    Duration,
    Cw20ReceiveMsg,
    Receiver,
    QueryMsg,
    Addr,
    ChainConfigResponse,
    Uint64,
} from "./RemoteTunnel.types";
export interface RemoteTunnelReadOnlyInterface {
    contractAddress: string;
    daoConfig: () => Promise<DaoConfig>;
    chainConfig: () => Promise<ChainConfigResponse>;
    ibcTransferChannels: ({
        limit,
        startAfter,
    }: {
        limit?: number;
        startAfter?: string;
    }) => Promise<IbcTransferChannels>;
    nextJobId: () => Promise<Uint64>;
}
export class RemoteTunnelQueryClient implements RemoteTunnelReadOnlyInterface {
    client: CosmWasmClient;
    contractAddress: string;

    constructor(client: CosmWasmClient, contractAddress: string) {
        this.client = client;
        this.contractAddress = contractAddress;
        this.daoConfig = this.daoConfig.bind(this);
        this.chainConfig = this.chainConfig.bind(this);
        this.ibcTransferChannels = this.ibcTransferChannels.bind(this);
        this.nextJobId = this.nextJobId.bind(this);
    }

    daoConfig = async (): Promise<DaoConfig> => {
        return this.client.queryContractSmart(this.contractAddress, {
            dao_config: {},
        });
    };
    chainConfig = async (): Promise<ChainConfigResponse> => {
        return this.client.queryContractSmart(this.contractAddress, {
            chain_config: {},
        });
    };
    ibcTransferChannels = async ({
        limit,
        startAfter,
    }: {
        limit?: number;
        startAfter?: string;
    }): Promise<IbcTransferChannels> => {
        return this.client.queryContractSmart(this.contractAddress, {
            ibc_transfer_channels: {
                limit,
                start_after: startAfter,
            },
        });
    };
    nextJobId = async (): Promise<Uint64> => {
        return this.client.queryContractSmart(this.contractAddress, {
            next_job_id: {},
        });
    };
}
export interface RemoteTunnelInterface extends RemoteTunnelReadOnlyInterface {
    contractAddress: string;
    sender: string;
    daoActions: (
        {
            msg,
        }: {
            msg: RemoteTunnelPacketMsg;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
    ibcTransfer: (
        {
            receiver,
        }: {
            receiver: Receiver;
        },
        fee?: number | StdFee | "auto",
        memo?: string,
        funds?: Coin[]
    ) => Promise<ExecuteResult>;
}
export class RemoteTunnelClient extends RemoteTunnelQueryClient implements RemoteTunnelInterface {
    override client: SigningCosmWasmClient;
    sender: string;
    override contractAddress: string;

    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, contractAddress);
        this.client = client;
        this.sender = sender;
        this.contractAddress = contractAddress;
        this.daoActions = this.daoActions.bind(this);
        this.ibcTransfer = this.ibcTransfer.bind(this);
    }

    daoActions = async (
        {
            msg,
        }: {
            msg: RemoteTunnelPacketMsg;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                dao_actions: {
                    msg,
                },
            },
            fee,
            memo,
            funds
        );
    };
    ibcTransfer = async (
        {
            receiver,
        }: {
            receiver: Receiver;
        },
        fee: number | StdFee | "auto" = "auto",
        memo?: string,
        funds?: Coin[]
    ): Promise<ExecuteResult> => {
        return await this.client.execute(
            this.sender,
            this.contractAddress,
            {
                ibc_transfer: {
                    receiver,
                },
            },
            fee,
            memo,
            funds
        );
    };
}
