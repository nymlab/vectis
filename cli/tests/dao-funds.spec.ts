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

describe("DAO Suite:", () => {
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

        const initialFunds = walletInitialFunds(hostChain);
        const { wallet_fee } = await factoryClient.fees();
        const totalFee: Number = Number(wallet_fee.amount) + Number(initialFunds.amount);

        const { amount: remainingBalance } = await userClient.getBalance(userClient.sender, remoteChain.feeToken);
        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                    },
                    relayers: [hostAccounts.relayer_1.address],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(hostChain),
            undefined,
            [coin(totalFee.toString(), hostChain.feeToken) as Coin]
        );

        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        const walletAddr = wallets[0][0];
        proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
        const { claim_fee } = await factoryClient.fees();
        await proxyClient.mintGovec(addrs.factoryAddr, claim_fee);
        await proxyClient.stakeGovec(addrs.govecAddr, addrs.stakingAddr, "2");

        // ensure all previously ibc messages have been relayed
        await relayerClient.relayAll();
        await delay(10000);
    });

    //    it("DAO can send funds from remote-tunnel to others", async () => {
    //        const funds = [{ amount: (1e7).toString(), denom: remoteChain.feeToken }];
    //        await userRemoteClient.sendTokens(userRemoteClient.sender, addrs.remoteTunnelAddr, funds, "auto");
    //        const { amount: rmtFactoryPreviousBalance } = await userRemoteClient.getBalance(
    //            addrs.remoteFactoryAddr,
    //            remoteChain.feeToken
    //        );
    //        const { amount: rmtTunnelPreviousBalance } = await userRemoteClient.getBalance(
    //            addrs.remoteFactoryAddr,
    //            remoteChain.feeToken
    //        );
    //
    //        console.log(relayerClient.channels);
    //
    //        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
    //            dispatch_action_on_remote_tunnel: {
    //                channel_id: relayerClient.channels.wasm?.src.channelId as string,
    //                job_id: 1,
    //                msg: {
    //                    dispatch_actions: {
    //                        msgs: [
    //                            {
    //                                bank: {
    //                                    send: {
    //                                        amount: funds,
    //                                        to_address: addrs.remoteFactoryAddr,
    //                                    },
    //                                },
    //                            },
    //                        ],
    //                    },
    //                },
    //            },
    //        });
    //
    //        await proxyClient.createProposal(
    //            daoClient.proposalAddr,
    //            "Send funds from remote tunnel",
    //            "Send funds from remote tunne",
    //            [msg]
    //        );
    //        await delay(10000);
    //
    //        const { proposals } = await daoClient.queryProposals();
    //        const proposalId = proposals.length;
    //
    //        await proxyClient.voteProposal(daoClient.proposalAddr, proposalId, "yes");
    //        await delay(10000);
    //
    //        await proxyClient.executeProposal(daoClient.proposalAddr, proposalId);
    //        await delay(10000);
    //        await relayerClient.relayAll();
    //
    //        const { amount: rmtFactoryCurrentBalance } = await userRemoteClient.getBalance(
    //            addrs.remoteFactoryAddr,
    //            remoteChain.feeToken
    //        );
    //        const { amount: rmtTunnelCurrentBalance } = await userRemoteClient.getBalance(
    //            addrs.remoteFactoryAddr,
    //            remoteChain.feeToken
    //        );
    //
    //        expect(+rmtTunnelPreviousBalance).toBe(+rmtTunnelCurrentBalance - +funds[0].amount);
    //        expect(+rmtFactoryPreviousBalance + +funds[0].amount).toBe(+rmtFactoryCurrentBalance);
    //    });
    //
    //    it("DAO can instruct remote-tunnel to stake / delegate", async () => {
    //        const funds = { amount: (1e7).toString(), denom: remoteChain.feeToken };
    //        await userRemoteClient.sendTokens(userRemoteClient.sender, addrs.remoteTunnelAddr, [funds], "auto");
    //        let balanceBeforeStaking = "0";
    //        const { validators } = await remoteQueryClient.staking.validators("BOND_STATUS_BONDED");
    //        const [validator] = validators;
    //        try {
    //            const delegations = await remoteQueryClient.staking.delegation(
    //                addrs.remoteTunnelAddr,
    //                validator.operatorAddress
    //            );
    //            balanceBeforeStaking = delegations?.delegationResponse?.balance?.amount || "0";
    //        } catch (err) {}
    //
    //        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
    //            dispatch_action_on_remote_tunnel: {
    //                channel_id: relayerClient.channels.wasm?.src.channelId as string,
    //                job_id: 3,
    //                msg: {
    //                    dispatch_actions: {
    //                        msgs: [
    //                            {
    //                                staking: {
    //                                    delegate: {
    //                                        amount: funds,
    //                                        validator: validator.operatorAddress,
    //                                    },
    //                                },
    //                            },
    //                        ],
    //                    },
    //                },
    //            },
    //        });
    //
    //        await proxyClient.createProposal(
    //            daoClient.proposalAddr,
    //            "Stake action from remote tunnel",
    //            "Stake action from remote tunnel",
    //            [msg]
    //        );
    //        await delay(10000);
    //
    //        const { proposals } = await daoClient.queryProposals();
    //        const proposalId = proposals.length;
    //
    //        await proxyClient.voteProposal(daoClient.proposalAddr, proposalId, "yes");
    //        await delay(10000);
    //
    //        await proxyClient.executeProposal(daoClient.proposalAddr, proposalId);
    //        await delay(10000);
    //        await relayerClient.relayAll();
    //
    //        await delay(10000);
    //        const resp = await remoteQueryClient.staking.delegation(addrs.remoteTunnelAddr, validator.operatorAddress);
    //        const balanceAfterStaking = resp.delegationResponse?.balance?.amount || NaN;
    //
    //        expect(+balanceBeforeStaking + +funds.amount).toBe(+balanceAfterStaking);
    //    });
    //
    //it("Test user send ibc to dao", async () => {
    //    await userRemoteClient.execute(
    //        userRemoteClient.sender,
    //        addrs.remoteTunnelAddr,
    //        {
    //            ibc_transfer: {
    //                receiver: { connection_id: relayerClient.connections.remoteConnection, addr: addrs.daoAddr },
    //            },
    //        },
    //        "auto",
    //        undefined,
    //        [coin(1e4, remoteChain.feeToken)]
    //    );
    //    await relayerClient.relayAll();
    //    await delay(10000);

    //    console.log("denoms: ", relayerClient.denoms);
    //});

    it("DAO can send funds from remote-tunnel to itself via contract and ibc-transfer module", async () => {
        const tosend = 1e7;
        const funds = { amount: (2 * tosend).toString(), denom: remoteChain.feeToken };
        await userRemoteClient.sendTokens(userRemoteClient.sender, addrs.remoteTunnelAddr, [funds], "auto");
        console.log("remote tunnel:", addrs.remoteTunnelAddr);

        const { amount: daoPreviousBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);
        console.log("denom: ", relayerClient.denoms.dest);
        console.log("previous balance dao: ", daoPreviousBalance);

        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 1111,
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
                                            timestamp: long
                                                .fromNumber(Math.floor(Date.now() / 1000) + 60)
                                                .multiply(1e12)
                                                .toString(),
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

        await proxyClient.createProposal(
            daoClient.proposalAddr,
            "Send funds from remote tunnel to DAO",
            "Send funds from remote tunne to DAO",
            [msg]
        );
        await delay(8000);

        const { proposals } = await daoClient.queryProposals();
        const proposalId = proposals.length;
        console.log("id: ", proposalId);

        await proxyClient.voteProposal(daoClient.proposalAddr, proposalId, "yes");
        await delay(8000);

        await proxyClient.executeProposal(daoClient.proposalAddr, proposalId);
        await delay(12000);
        await relayerClient.relayAll();

        await delay(8000);
        await relayerClient.relayAll();
        const { amount: daoCurrentBalance } = await userClient.getBalance(addrs.daoAddr, relayerClient.denoms.dest);
        const balances = await queryClient.bank.allBalances(addrs.daoAddr);
        console.log("final balances: ", balances);

        expect(+daoPreviousBalance + 2 * tosend).toBe(+daoCurrentBalance);
    });

    afterAll(async () => {
        await proxyClient.unstakeGovec(addrs.stakingAddr, "2");
        await proxyClient.burnGovec(addrs.govecAddr);
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe("0");
        adminClient?.disconnect();
    });
});
