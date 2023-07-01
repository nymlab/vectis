import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { hubDeployReportPath, hostChain, hubUploadReportPath } from "../utils/constants";
import { CWClient, FactoryClient, ProxyClient } from "../clients";
import { walletInitialFunds } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import { VectisHubChainContractsAddrs } from "../interfaces/contracts";
import { Coin } from "../interfaces/Factory.types";
import { createSingleProxyWallet } from "./mocks/proxyWallet";

/**
 * This suite tests Factory contract methods
 */
describe("Factory Suite: ", () => {
    let client: CosmWasmClient;
    let userClient: CWClient;
    let addrs: VectisHubChainContractsAddrs;
    let proxyCodeId: number;

    let factoryClient: FactoryClient;
    let proxyClient: ProxyClient;
    beforeAll(async () => {
        const codes = await import(hubUploadReportPath);
        addrs = await import(hubDeployReportPath);
        proxyCodeId = codes.proxy.codeId;
        userClient = await CWClient.connectHostWithAccount("user");
        client = await CosmWasmClient.connect(hostChain.rpcUrl);

        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.Factory);
    });

    it("Should store Proxy code id in Factory contract", async () => {
        const codeId = await factoryClient.codeId({ ty: "proxy" });
        expect(codeId).toEqual(proxyCodeId);
    });

    it("should be able to create a proxy wallet", async () => {
        const totalWalletBeforeCreation = await factoryClient.totalCreated();
        const walletAddr = await createSingleProxyWallet(factoryClient, "host");
        proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr!);
        const totalWalletAfterCreation = await factoryClient.totalCreated();
        const balance = await userClient.getBalance(proxyClient.contractAddress, hostChain.feeToken);
        const initialFunds = walletInitialFunds(hostChain);
        expect(balance).toEqual(initialFunds);
        expect(totalWalletBeforeCreation + 1).toBe(totalWalletAfterCreation);
    });

    it("Should get correct balance in proxy wallet", async () => {
        const initialFunds = walletInitialFunds(hostChain);
        const balance = await client.getBalance(proxyClient.contractAddress, hostChain.feeToken);
        expect(balance).toEqual(initialFunds);
    });

    afterAll(() => {
        userClient?.disconnect();
        client?.disconnect();
    });
});
