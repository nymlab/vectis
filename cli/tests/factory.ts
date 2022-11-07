import { coin } from "@cosmjs/stargate";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { uploadReportPath } from "@vectis/core/utils/constants";
import { CWClient, FactoryClient, GovecClient } from "@vectis/core/clients";
import {
    HOST_ACCOUNTS,
    HOST_CHAIN,
    getInitialFactoryBalance,
    walletInitialFunds,
    getDefaultWalletCreationFee,
} from "./mocks/constants";
import { FactoryT } from "@vectis/types";

/**
 * This suite tests Factory contract methods
 */
describe("Factory Suite: ", () => {
    let adminClient: CWClient;
    let client: CosmWasmClient;
    let proxyCodeId: number;

    let factoryClient: FactoryClient;
    let proxyWalletAddress: FactoryT.Addr;

    beforeAll(async () => {
        const { host } = await import(uploadReportPath);
        const { proxyRes, factoryRes, govecRes, multisigRes } = host;
        proxyCodeId = proxyRes.codeId;
        adminClient = await CWClient.connectWithAccount("juno_localnet", "admin");
        client = await CosmWasmClient.connect(HOST_CHAIN.rpcUrl);

        factoryClient = await FactoryClient.instantiate(
            adminClient,
            factoryRes.codeId,
            FactoryClient.createFactoryInstMsg("juno_localnet", proxyRes.codeId, multisigRes.codeId),
            [getInitialFactoryBalance(HOST_CHAIN)]
        );

        let govecClient = await GovecClient.instantiate(adminClient, govecRes.codeId, {
            factory: factoryClient.contractAddress,
            initial_balances: [],
        });

        await factoryClient.updateGovecAddr({ addr: govecClient.contractAddress });
        const govec = await factoryClient.govecAddr();

        expect(govec).toEqual(govecClient.contractAddress);
    });

    it("Should have correct funds in Factory contract", async () => {
        const fund = await client.getBalance(factoryClient.contractAddress, HOST_CHAIN.feeToken);
        expect(fund).toEqual(getInitialFactoryBalance(HOST_CHAIN));
    });

    it("Should store Proxy code id in Factory contract", async () => {
        const codeId = await factoryClient.codeId({ ty: "proxy" });
        expect(codeId).toEqual(proxyCodeId);
    });

    it("Should create a new proxy wallet with multisig", async () => {
        const walletCreationFee = await factoryClient.fee();
        const initialFunds = walletInitialFunds(HOST_CHAIN);
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

        const newWalletRes = await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: HOST_ACCOUNTS.user.address,
                    guardians: {
                        addresses: [HOST_ACCOUNTS.guardian_1.address, HOST_ACCOUNTS.guardian_2.address],
                        guardians_multisig: {
                            threshold_absolute_count: 1,
                            multisig_initial_funds: [],
                        },
                    },
                    relayers: [HOST_ACCOUNTS.relayer_1.address, HOST_ACCOUNTS.relayer_2.address],
                    proxy_initial_funds: [initialFunds as FactoryT.Coin],
                    label: "initial label",
                },
            },
            getDefaultWalletCreationFee(HOST_CHAIN),
            undefined,
            [coin(totalFee.toString(), HOST_CHAIN.feeToken) as FactoryT.Coin]
        );

        const wasmEvent = newWalletRes.logs[0].events.length;
        const wasmEventAttributes = newWalletRes.logs[0].events[wasmEvent - 1].attributes;
        proxyWalletAddress = wasmEventAttributes.find((event) => event.key === "proxy_address")?.value!;

        expect(proxyWalletAddress).toBeTruthy();
    });

    it("Should have the wallet pending to claim the govec token", async () => {
        // By calling 'wallets'
        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        expect(wallets.length).toEqual(1);
        expect(wallets[0][0]).toEqual(proxyWalletAddress);
    });

    it("Should get correct balance in proxy wallet", async () => {
        const initialFunds = walletInitialFunds(HOST_CHAIN);
        const balance = await client.getBalance(proxyWalletAddress, HOST_CHAIN.feeToken);
        expect(balance).toEqual(initialFunds);
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
