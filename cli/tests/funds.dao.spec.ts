import { FactoryClient, GovecClient, CWClient, DaoClient, ProxyClient, RelayClient } from "../clients";
import { hostAccounts, hostChain, remoteChain } from "../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import {
    BankExtension,
    coin,
    QueryClient,
    setupBankExtension,
    setupStakingExtension,
    SigningStargateClient,
    StakingExtension,
} from "@cosmjs/stargate";
import { Coin } from "../interfaces/Factory.types";
import { DaoTunnelClient } from "../interfaces";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { deployReportPath } from "../utils/constants";
import { delay } from "../utils/promises";
import { Tendermint34Client } from "@cosmjs/tendermint-rpc";
import long from "long";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite for DAO Funds:", () => {
    let addrs: VectisDaoContractsAddrs;
    let adminClient: CWClient;
    let userClient: CWClient;
    let userRemoteClient: CWClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let factoryClient: FactoryClient;
    let relayerClient: RelayClient;
    let queryClient: QueryClient & StakingExtension & BankExtension;
    let remoteQueryClient: QueryClient & StakingExtension & BankExtension;
    SigningStargateClient;
    beforeAll(async () => {
        queryClient = await QueryClient.withExtensions(
            await Tendermint34Client.connect(hostChain.rpcUrl),
            setupStakingExtension,
            setupBankExtension
        );
        remoteQueryClient = await QueryClient.withExtensions(
            await Tendermint34Client.connect(remoteChain.rpcUrl),
            setupStakingExtension,
            setupBankExtension
        );
        addrs = await import(deployReportPath);
        adminClient = await CWClient.connectHostWithAccount("admin");
        userClient = await CWClient.connectHostWithAccount("user");
        userRemoteClient = await CWClient.connectRemoteWithAccount("user");
        govecClient = new GovecClient(adminClient, adminClient.sender, addrs.govecAddr);
        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.factoryAddr);
        daoClient = new DaoClient(adminClient, {
            daoAddr: addrs.daoAddr,
            proposalAddr: addrs.proposalAddr,
            stakingAddr: addrs.stakingAddr,
            voteAddr: addrs.voteAddr,
        });
        relayerClient = new RelayClient();
        await relayerClient.connect();
        await relayerClient.loadChannels();
    });

    it("DAO can instruct remote-tunnel to stake / delegate", async () => {
        const funds = { amount: (1e7).toString(), denom: remoteChain.feeToken };
        await userRemoteClient.sendTokens(userRemoteClient.sender, addrs.remoteTunnelAddr, [funds], "auto");
        let balanceBeforeStaking = "0";
        const { validators } = await remoteQueryClient.staking.validators("BOND_STATUS_BONDED");
        const [validator] = validators;
        try {
            const delegations = await remoteQueryClient.staking.delegation(
                addrs.remoteTunnelAddr,
                validator.operatorAddress
            );
            balanceBeforeStaking = delegations?.delegationResponse?.balance?.amount || "0";
        } catch (err) {}

        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 3,
                msg: {
                    dispatch_actions: {
                        msgs: [
                            {
                                staking: {
                                    delegate: {
                                        amount: funds,
                                        validator: validator.operatorAddress,
                                    },
                                },
                            },
                        ],
                    },
                },
            },
        });

        await daoClient.executeAdminMsg(msg);
        await delay(10000);
        await relayerClient.relayAll();

        await delay(10000);
        const resp = await remoteQueryClient.staking.delegation(addrs.remoteTunnelAddr, validator.operatorAddress);
        const balanceAfterStaking = resp.delegationResponse?.balance?.amount || NaN;

        expect(+balanceBeforeStaking + +funds.amount).toBe(+balanceAfterStaking);
    });

    it("Test remote proxy send funds to dao-chain via IBC transfer on remote tunnel", async () => {
        const { amount: daoPreviousBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);

        await userRemoteClient.execute(
            userRemoteClient.sender,
            addrs.remoteTunnelAddr,
            {
                ibc_transfer: {
                    receiver: { connection_id: relayerClient.connections.remoteConnection, addr: addrs.daoAddr },
                },
            },
            "auto",
            undefined,
            [coin(1e4, remoteChain.feeToken)]
        );
        await relayerClient.relayAll();
        await delay(10000);

        const { amount: daoCurrentBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);

        expect(+daoPreviousBalance + 1e4).toBe(+daoCurrentBalance);
    });

    it("DAO can send funds from remote-tunnel to itself via contract and ibc-transfer module", async () => {
        const tosend = 1e5;
        const funds = { amount: (2 * tosend).toString(), denom: remoteChain.feeToken };
        const result = await userRemoteClient.sendTokens(
            userRemoteClient.sender,
            addrs.remoteTunnelAddr,
            [funds],
            "auto"
        );

        const { amount: daoPreviousBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);

        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 111,
                msg: {
                    dispatch_actions: {
                        msgs: [
                            {
                                wasm: {
                                    execute: {
                                        contract_addr: addrs.remoteTunnelAddr,
                                        funds: [coin(tosend, remoteChain.feeToken)],
                                        msg: toCosmosMsg({
                                            ibc_transfer: {
                                                receiver: {
                                                    connection_id: relayerClient.connections.remoteConnection,
                                                    addr: addrs.daoAddr,
                                                },
                                            },
                                        }),
                                    },
                                },
                            },
                            {
                                ibc: {
                                    transfer: {
                                        amount: coin(tosend, remoteChain.feeToken),
                                        channel_id: relayerClient.channels.transfer?.dest.channelId,
                                        timeout: {
                                            timestamp: "7718596707569197056",
                                        },
                                        to_address: daoClient.daoAddr,
                                    },
                                },
                            },
                        ],
                    },
                },
            },
        });

        await daoClient.executeAdminMsg(msg);
        await relayerClient.relayAll();

        await delay(10000);
        await relayerClient.relayAll();

        const { amount: daoCurrentBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);

        expect(+daoPreviousBalance + 2 * tosend).toBe(+daoCurrentBalance);
    });

    afterAll(async () => {
        await daoClient.executeRemoveAdmin();

        try {
            await daoClient.queryAdmin();
            expect(false);
        } catch (err) {}
        adminClient?.disconnect();
    });
});
