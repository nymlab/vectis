import { CWClient, FactoryClient, GovecClient } from "../clients";
import { coin } from "@cosmjs/stargate";
import { deployReportPath, remoteChain, remoteAccounts } from "../utils/constants";
import { ProxyClient } from "../interfaces";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { Coin, Expiration } from "../interfaces/Factory.types";
import RelayerClient from "../clients/relayer";
import { toCosmosMsg } from "../utils/enconding";

describe("Remote Factory Suite:", () => {
    let userClient: CWClient;
    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    const relayerClient = new RelayerClient();
    beforeAll(async () => {
        const { remoteFactoryAddr, govecAddr } = await import(deployReportPath);
        await relayerClient.connect();
        userClient = await CWClient.connectRemoteWithAccount("user");
        factoryClient = new FactoryClient(userClient, userClient.sender, remoteFactoryAddr);
        govecClient = new GovecClient(userClient, userClient.sender, govecAddr);
    });
    it("should create a proxy wallet", async () => {
        const initialFunds = walletInitialFunds(remoteChain);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

        const totalWalletBeforeCreation = await factoryClient.totalCreated();

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
        proxyClient = new ProxyClient(userClient, userClient.sender, wallets[0][0]);

        const totalWalletAfterCreation = await factoryClient.totalCreated();

        expect(totalWalletBeforeCreation + 1).toBe(totalWalletAfterCreation);
    });
    it("should create a proxy wallet and it would", async () => {
        let res = await factoryClient.unclaimedGovecWallets({});
        let targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);

        expect(targetWallet).toBeDefined();

        await proxyClient.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: factoryClient.contractAddress,
                            funds: [],
                            msg: toCosmosMsg({ claim_govec: {} }),
                        },
                    },
                },
            ],
        });

        await relayerClient.relayAll();
        res = await factoryClient.unclaimedGovecWallets({});
        targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);
        expect(targetWallet).toBeUndefined();
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe("1");
    });
});