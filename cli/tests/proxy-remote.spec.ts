import { coin } from "@cosmjs/amino";
import {
    BankExtension,
    QueryClient,
    setupBankExtension,
    setupStakingExtension,
    StakingExtension,
} from "@cosmjs/stargate";
import { Tendermint34Client } from "@cosmjs/tendermint-rpc";
import { CWClient, GovecClient, ProxyClient, RelayerClient } from "../clients";
import RemoteProxyClient from "../clients/remote-proxy";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { Coin } from "../interfaces/Factory.types";
import { RemoteFactoryClient } from "../interfaces/RemoteFactory.client";
import { deployReportPath, hostChain, remoteAccounts, remoteChain } from "../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { delay } from "../utils/promises";

describe("Proxy Remote Suite: ", () => {
    let addrs: VectisDaoContractsAddrs;
    let userClient: CWClient;
    let hostUserClient: CWClient;
    let factoryClient: RemoteFactoryClient;
    let proxyClient: RemoteProxyClient;
    let govecClient: GovecClient;
    let hostQueryClient: QueryClient & StakingExtension & BankExtension;
    const relayerClient = new RelayerClient();
    beforeAll(async () => {
        addrs = await import(deployReportPath);
        await relayerClient.connect();
        await relayerClient.loadChannels();
        userClient = await CWClient.connectRemoteWithAccount("user");
        hostUserClient = await CWClient.connectHostWithAccount("user");
        factoryClient = new RemoteFactoryClient(userClient, userClient.sender, addrs.remoteFactoryAddr);
        govecClient = new GovecClient(hostUserClient, hostUserClient.sender, addrs.govecAddr);
        hostQueryClient = await QueryClient.withExtensions(
            await Tendermint34Client.connect(hostChain.rpcUrl),
            setupStakingExtension,
            setupBankExtension
        );

        const initialFunds = walletInitialFunds(remoteChain);
        const { wallet_fee } = await factoryClient.fees();
        const totalFee: Number = Number(wallet_fee.amount) + Number(initialFunds.amount);

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [remoteAccounts.guardian_1.address, remoteAccounts.guardian_2.address],
                    },
                    relayers: [remoteAccounts.relayer_1.address],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(remoteChain),
            undefined,
            [coin(totalFee.toString(), remoteChain.feeToken) as Coin]
        );

        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        proxyClient = new RemoteProxyClient(userClient, userClient.sender, wallets[0][0]);
    });

    it("should be able to do ibc transfer", async () => {
        const amount = "1000000";
        const account = await CWClient.generateRandomAccount("juno");
        const [{ address }] = await account.getAccounts();
        const previousBalance = await hostUserClient.getBalance(address, relayerClient.denoms.dest);

        expect(previousBalance.amount).toBe("0");

        await proxyClient.sendIbcTokens(addrs.remoteTunnelAddr, address, relayerClient.connections.remoteConnection, [
            coin(amount, remoteChain.feeToken) as Coin,
        ]);

        await relayerClient.relayAll();

        const currentBalance = await hostUserClient.getBalance(address, relayerClient.denoms.dest);

        expect(currentBalance.amount).toBe(amount);
    });

    it("should be able to mint govec", async () => {
        const { claim_fee } = await factoryClient.fees();
        await proxyClient.mintGovec(addrs.remoteFactoryAddr, claim_fee);
        await relayerClient.relayAll();
        await proxyClient.executeProposal;
        const { accounts } = await govecClient.allAccounts({});
        expect(accounts.includes(proxyClient.contractAddress)).toBeTruthy();
    });

    it("should be able to stake", async () => {
        await proxyClient.stakeGovec(addrs.remoteTunnelAddr, addrs.stakingAddr, "2");
        await relayerClient.relayAll();
        const { value } = await hostUserClient.queryContractSmart(addrs.stakingAddr, {
            staked_value: { address: proxyClient.contractAddress },
        });
        expect(value).toBe("2");
    });

    it("should be able to unstake", async () => {
        const resp = await proxyClient.unstakeGovec(addrs.remoteTunnelAddr, "2");
        console.log(resp);
        await relayerClient.relayAll();
        await delay(10000);
        const { value } = await hostUserClient.queryContractSmart(addrs.stakingAddr, {
            staked_value: { address: proxyClient.contractAddress },
        });
        expect(value).toBe("0");
    });
});
