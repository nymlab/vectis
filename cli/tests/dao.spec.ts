import { FactoryClient, GovecClient, CWClient, DaoClient } from "../clients";
import { hostAccounts, hostChain } from "../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { coin } from "@cosmjs/stargate";
import { Coin, Expiration } from "../interfaces/Factory.types";
import { ProxyClient } from "../interfaces";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { deployReportPath } from "../utils/constants";
import { toCosmosMsg } from "../utils/enconding";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite:", () => {
    let addrs: VectisDaoContractsAddrs;
    let adminClient: CWClient;
    let userClient: CWClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let factoryClient: FactoryClient;

    beforeAll(async () => {
        addrs = await import(deployReportPath);
        adminClient = await CWClient.connectHostWithAccount("admin");
        userClient = await CWClient.connectHostWithAccount("user");
        govecClient = new GovecClient(adminClient, adminClient.sender, addrs.govecAddr);
        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.factoryAddr);
        daoClient = new DaoClient(adminClient, {
            daoAddr: addrs.daoAddr,
            proposalAddr: addrs.proposalAddr,
            stakingAddr: addrs.stakingAddr,
            voteAddr: addrs.voteAddr,
        });
    });

    it("Host Factory should be able to instantiate a proxy wallet", async () => {
        const initialFunds = walletInitialFunds(hostChain);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

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
        proxyClient = new ProxyClient(userClient, userClient.sender, wallets[0][0]);
    });

    it("Host Proxy should be able to execute transactions and Host Factory should allow to claim govec to new proxy wallets", async () => {
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

        res = await factoryClient.unclaimedGovecWallets({});
        targetWallet = res.wallets.find(([w]: [string, Expiration]) => w === proxyClient.contractAddress);
        expect(targetWallet).toBeUndefined();
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe("1");
    });

    it("DAO should be the only one authorized for communicate with dao_tunnel", async () => {
        const msgAuthorize = {
            add_approved_controller: {
                connection_id: 1,
                port_id: `wasm.address`,
            },
        };

        await adminClient
            .execute(adminClient.sender, addrs.daoTunnelAddr, msgAuthorize, "auto")
            .catch((err: Error) => expect(err).toBeInstanceOf(Error));
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
