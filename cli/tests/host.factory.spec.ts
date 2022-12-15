import { coin } from "@cosmjs/stargate";
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { deployReportPath, hostAccounts, hostChain, uploadReportPath } from "../utils/constants";
import { CWClient, FactoryClient, GovecClient, ProxyClient } from "../clients";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { toCosmosMsg } from "../utils/enconding";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { Coin, Expiration } from "../interfaces/Factory.types";

/**
 * This suite tests Factory contract methods
 */
describe("Factory Suite: ", () => {
    let client: CosmWasmClient;
    let userClient: CWClient;
    let addrs: VectisDaoContractsAddrs;
    let proxyCodeId: number;

    let factoryClient: FactoryClient;
    let govecClient: GovecClient;
    let proxyClient: ProxyClient;
    beforeAll(async () => {
        const { host } = await import(uploadReportPath);
        addrs = await import(deployReportPath);
        proxyCodeId = host.proxyRes.codeId;
        userClient = await CWClient.connectHostWithAccount("user");
        client = await CosmWasmClient.connect(hostChain.rpcUrl);

        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.factoryAddr);

        govecClient = new GovecClient(userClient, userClient.sender, addrs.govecAddr);
    });

    it("Should store Proxy code id in Factory contract", async () => {
        const codeId = await factoryClient.codeId({ ty: "proxy" });
        expect(codeId).toEqual(proxyCodeId);
    });

    it("should be able to create a proxy wallet", async () => {
        const initialFunds = walletInitialFunds(hostChain);
        const { wallet_fee } = await factoryClient.fees();
        const totalFee: Number = Number(wallet_fee.amount) + Number(initialFunds.amount);
        console.log("total fee: ", totalFee);
        console.log("init: ", initialFunds);
        console.log("wallet: ", wallet_fee);

        const totalWalletBeforeCreation = await factoryClient.totalCreated();
        let wallet: string | null;
        let { wallets: oldWallets } = await factoryClient.unclaimedGovecWallets({});

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    controller_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                    },
                    relayers: [],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(hostChain),
            undefined,
            [coin(totalFee.toString(), hostChain.feeToken) as Coin]
        );

        let { wallets: newWallets } = await factoryClient.unclaimedGovecWallets({});

        let oldAddrs = oldWallets.map((s, e) => s[0]);
        let newAddrs = newWallets.map((s, e) => s[0]);

        for (let addr of newAddrs) {
            if (!oldAddrs.includes(addr)) {
                wallet = addr;
            }
        }

        proxyClient = new ProxyClient(userClient, userClient.sender, wallet!);
        const info = await proxyClient.info();
        expect(info.controller_addr).toEqual(userClient.sender);
        const totalWalletAfterCreation = await factoryClient.totalCreated();
        expect(totalWalletBeforeCreation + 1).toBe(totalWalletAfterCreation);
    });

    it("Should get correct balance in proxy wallet", async () => {
        const initialFunds = walletInitialFunds(hostChain);
        const balance = await client.getBalance(proxyClient.contractAddress, hostChain.feeToken);
        expect(balance).toEqual(initialFunds);
    });

    it("should not allow to claim govec without fee ", async () => {
        let res = await factoryClient.unclaimedGovecWallets({});
        let targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);

        expect(targetWallet).toBeDefined();

        try {
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
        } catch (err) {
            expect(err).toBeDefined();
        }
    });

    it("should allow to claim govec to new proxy wallets", async () => {
        let res = await factoryClient.unclaimedGovecWallets({});
        const initDAOBalance = (await client.getBalance(addrs.daoAddr, hostChain.feeToken)) as Coin;
        const { claim_fee } = await factoryClient.fees();
        let targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);

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

        res = await factoryClient.unclaimedGovecWallets({});
        targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);
        expect(targetWallet).toBeUndefined();
        const mintAmount = await govecClient.mintAmount();
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe(mintAmount);

        const finalDAOBalance = (await client.getBalance(addrs.daoAddr, hostChain.feeToken)) as Coin;

        let diff = +finalDAOBalance.amount - +initDAOBalance.amount;
        expect(diff).toEqual(+claim_fee.amount);
    });

    afterAll(() => {
        userClient?.disconnect();
        client?.disconnect();
    });
});
