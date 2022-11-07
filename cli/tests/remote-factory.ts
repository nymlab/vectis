import { CWClient, FactoryClient } from "@vectis/core/clients";
import { coin } from "@cosmjs/stargate";
import { deployReportPath } from "@vectis/core/utils/constants";
import { ProxyClient } from "@vectis/types";
import { getDefaultWalletCreationFee, REMOTE_ACCOUNTS, REMOTE_CHAIN, walletInitialFunds } from "./mocks/constants";
import { Coin } from "@vectis/types/contracts/Factory.types";
import RelayerClient from "@vectis/core/clients/relayer";

describe("Remote Factory Suite:", () => {
    let userClient: CWClient;
    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    const relayerClient = new RelayerClient("juno_localnet", "wasm_localnet");
    beforeAll(async () => {
        const { remoteFactoryAddr } = await import(deployReportPath);
        await relayerClient.recoverConnection();
        userClient = await CWClient.connectWithAccount("wasm_localnet", "user");
        console.log(remoteFactoryAddr);
        factoryClient = new FactoryClient(userClient, userClient.sender, remoteFactoryAddr);
    });
    it("should create a proxy wallet", async () => {
        const initialFunds = walletInitialFunds(REMOTE_CHAIN);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

        const totalWalletBeforeCreation = await factoryClient.totalCreated();

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [REMOTE_ACCOUNTS.guardian_1.address, REMOTE_ACCOUNTS.guardian_2.address],
                    },
                    relayers: [REMOTE_ACCOUNTS.relayer_1.address],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(REMOTE_CHAIN),
            undefined,
            [coin(totalFee.toString(), REMOTE_CHAIN.feeToken) as Coin]
        );

        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        proxyClient = new ProxyClient(userClient, userClient.sender, wallets[0][0]);

        const totalWalletAfterCreation = await factoryClient.totalCreated();

        expect(totalWalletBeforeCreation + 1).toBe(totalWalletAfterCreation);
    });
    it("should create a proxy wallet and it would", async () => {
        await factoryClient.claimGovec();
        await relayerClient.relayAll();
        const { wallets: unClaimedGovecWallets } = await factoryClient.unclaimedGovecWallets({});
        const walletShouldNotBeHere = unClaimedGovecWallets.find((wallet) => wallet[0] === proxyClient.contractAddress);
        expect(walletShouldNotBeHere).toBeUndefined();
    });
});
