import { FactoryClient, GovecClient, CWClient, DaoClient } from "@vectis/core/clients";
import { marketingDescription, marketingProject } from "@vectis/core/clients/govec";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite:", () => {
    const addrs = global.contracts;
    let adminClient: CWClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let factoryClient: FactoryClient;

    beforeAll(async () => {
        adminClient = await CWClient.connectWithAccount("juno_localnet", "admin");
        govecClient = new GovecClient(adminClient, adminClient.sender, addrs.govecAddr);
        factoryClient = new FactoryClient(adminClient, adminClient.sender, addrs.factoryAddr);
        daoClient = new DaoClient(adminClient, {
            daoAddr: addrs.daoAddr,
            proposalAddr: addrs.proposalAddr,
            stakingAddr: addrs.stakingAddr,
            voteAddr: addrs.voteAddr,
        });
    });

    it("Admin should have no govec tokens", async () => {
        try {
            await govecClient.balance({ address: adminClient.sender });
            expect(false).toBeTruthy;
        } catch (error) {
            expect(error).toBeInstanceOf(Error);
        }
        const tokenInfo = await govecClient.tokenInfo();
        expect(tokenInfo.total_supply).toEqual("0");
    });

    it("Factory, Govec, cw20_stake, cw20_stake_balance_voting, Proposal Contracts should have DAO as admin", async () => {
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

    it("Dao should have executed 4 proposals", async () => {
        const { proposals } = await daoClient.queryProposals();
        expect(proposals.length).toEqual(4);
    });

    it("download logo shouldn't return an error logo not found", async () => {
        await govecClient.downloadLogo().catch((err) => expect(err).toBeInstanceOf(Error));
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
