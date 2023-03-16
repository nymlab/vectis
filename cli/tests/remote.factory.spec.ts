import { CWClient, FactoryClient, GovecClient } from "../clients";
import { deployReportPath, remoteChain } from "../utils/constants";
import { ProxyClient } from "../interfaces";
import { walletInitialFunds } from "../utils/fees";
import { Coin, Expiration } from "../interfaces/Factory.types";
import RelayerClient from "../clients/relayer";
import { toCosmosMsg } from "../utils/enconding";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { createSingleProxyWallet } from "./mocks/proxyWallet";

describe("Remote Factory Suite:", () => {
    let userClient: CWClient;
    let hostUserClient: CWClient;
    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let addrs: VectisDaoContractsAddrs;
    let wallet: string | null;
    const relayerClient = new RelayerClient();
    beforeAll(async () => {
        const { remoteFactoryAddr, govecAddr } = await import(deployReportPath);
        await relayerClient.connect();
        addrs = await import(deployReportPath);
        userClient = await CWClient.connectRemoteWithAccount("user");
        hostUserClient = await CWClient.connectHostWithAccount("user");
        factoryClient = new FactoryClient(userClient, userClient.sender, remoteFactoryAddr);
        govecClient = new GovecClient(hostUserClient, hostUserClient.sender, govecAddr);
    });
    it("should create a proxy wallet", async () => {
        const totalWalletBeforeCreation = await factoryClient.totalCreated();
        const walletAddr = await createSingleProxyWallet(factoryClient, "remote");
        proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr!);
        const totalWalletAfterCreation = await factoryClient.totalCreated();
        const balance = await userClient.getBalance(proxyClient.contractAddress, remoteChain.feeToken);
        const initialFunds = walletInitialFunds(remoteChain);
        expect(balance).toEqual(initialFunds);
        expect(totalWalletBeforeCreation + 1).toBe(totalWalletAfterCreation);
    });

    it("proxy should be able to mint govec tokens", async () => {
        let res = await factoryClient.unclaimedGovecWallets({});
        const initTunnelBalance = (await userClient.getBalance(addrs.remoteTunnelAddr, remoteChain.feeToken)) as Coin;
        let targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);

        const { claim_fee } = await factoryClient.fees();
        expect(targetWallet).toBeDefined();

        await proxyClient.execute({
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: factoryClient.contractAddress,
                            funds: [claim_fee],
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
        expect(balance).toBe("2");

        const finalTunnelBalance = (await userClient.getBalance(addrs.remoteTunnelAddr, remoteChain.feeToken)) as Coin;
        let diff = +finalTunnelBalance.amount - +initTunnelBalance.amount;
        expect(diff).toEqual(+claim_fee.amount);
    });
});
