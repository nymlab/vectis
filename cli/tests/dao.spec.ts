import { FactoryClient, GovecClient, CWClient, DaoClient } from "@vectis/core/clients";
import { marketingDescription, marketingProject } from "@vectis/core/clients/govec";
import { getDefaultWalletCreationFee, HOST_ACCOUNTS, HOST_CHAIN, walletInitialFunds } from "./mocks/constants";
import { coin } from "@cosmjs/stargate";
import { Coin } from "@vectis/types/contracts/Factory.types";
import { ProxyClient } from "@vectis/types";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite:", () => {
    const addrs = global.contracts;
    let adminClient: CWClient;
    let userClient: CWClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let factoryClient: FactoryClient;

    beforeAll(async () => {
        adminClient = await CWClient.connectWithAccount("juno_localnet", "admin");
        userClient = await CWClient.connectWithAccount("juno_localnet", "user");
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
        const initialFunds = walletInitialFunds(HOST_CHAIN);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [HOST_ACCOUNTS.guardian_1.address, HOST_ACCOUNTS.guardian_2.address],
                    },
                    relayers: [HOST_ACCOUNTS.relayer_1.address],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(HOST_CHAIN),
            undefined,
            [coin(totalFee.toString(), HOST_CHAIN.feeToken) as Coin]
        );

        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        proxyClient = new ProxyClient(userClient, userClient.sender, wallets[wallets.length - 1][0]);
    });

    it("Host Factory should allow to claim govec to new proxy wallets", async () => {
        let res = await factoryClient.unclaimedGovecWallets({});
        let targetWallet = res.wallets.find(([w]) => w === proxyClient.contractAddress);
        expect(targetWallet).toBeDefined();

        await factoryClient.claimGovec();

        res = await factoryClient.unclaimedGovecWallets({});
        targetWallet = res.wallets.find(([w]) => w === proxyClient.contractAddress);
        expect(targetWallet).toBeUndefined();
        const { balance } = await govecClient.balance({ address: proxyClient.contractAddress });
        expect(balance).toBe("1");
    });

    it("Deployer account should have no tokens", async () => {
        const tokenInfo = await govecClient.tokenInfo();
        expect(tokenInfo.total_supply).toEqual("0");
    });

    it("DAO should not has admin", async () => {
        const contract = await adminClient.getContract(addrs.daoAddr);
        expect(contract.admin).toEqual(undefined);
    });

    it("Govec, dao_tunnel, host_factory, cw20_stake, cw20_stake_balance_voting, Proposal Contracts should have DAO as admin", async () => {
        let contract = await adminClient.getContract(addrs.factoryAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
        contract = await adminClient.getContract(addrs.govecAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
        contract = await adminClient.getContract(addrs.stakingAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
        contract = await adminClient.getContract(addrs.voteAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
        contract = await adminClient.getContract(addrs.proposalAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
        contract = await adminClient.getContract(addrs.daoTunnelAddr);
        expect(contract.admin).toEqual(addrs.daoAddr);
    });

    it("DAO should have executed 4 proposals", async () => {
        const { proposals } = await daoClient.queryProposals();
        expect(proposals.length).toEqual(4);
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
            .catch((err) => expect(err).toBeInstanceOf(Error));
    });

    it("Govec should have factory addr and dao_tunnel addr", async () => {
        const m = await govecClient.minters();
        expect(m.minters).toEqual([addrs.daoTunnelAddr, addrs.factoryAddr]);
    });

    it("Govec should have daoAddr as the dao", async () => {
        const dao = await govecClient.dao();
        expect(dao).toEqual(addrs.daoAddr);
    });

    it("Govec should have stakingAddr as the stakingAddr", async () => {
        const staking = await govecClient.staking();
        expect(staking).toEqual(addrs.stakingAddr);
    });

    it("Govec should be set on the factory", async () => {
        const g = await factoryClient.govecAddr();
        expect(g).toEqual(addrs.govecAddr);
    });

    it("Govec should have have vectis project and description", async () => {
        const marketingInfo = await govecClient.marketingInfo();
        expect(marketingInfo.project).toEqual(marketingProject);
        expect(marketingInfo.description).toEqual(marketingDescription);
    });

    it("download logo shouldn't return an error logo not found", async () => {
        await govecClient.downloadLogo().catch((err) => expect(err).toBeInstanceOf(Error));
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
