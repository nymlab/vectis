import { GovecClient } from "@vectis/types/contracts/GovecContract";
import { FactoryClient } from "@vectis/types/contracts/FactoryContract";
import { adminAddr, addrPrefix, adminMnemonic } from "@vectis/core/utils/constants";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { createSigningClient } from "@vectis/core/services/cosmwasm";
import { deploy } from "@vectis/core/utils/dao-deploy";
import { VectisDaoContractsAddrs } from "@vectis/core/interfaces/dao";
import { marketingDescription, marketingProject } from "@vectis/core/services/govec";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */
describe("DAO Suite: ", () => {
    let adminClient: SigningCosmWasmClient;
    let govecClient: GovecClient;
    let factoryClient: FactoryClient;
    let addrs: VectisDaoContractsAddrs;

    beforeAll(async () => {
        addrs = await deploy();
        adminClient = await createSigningClient(adminMnemonic, addrPrefix);
        govecClient = new GovecClient(adminClient, adminAddr, addrs.govecAddr);
        factoryClient = new FactoryClient(adminClient, adminAddr, addrs.factoryAddr);
    });

    it("Admin should have no govec tokens", async () => {
        try {
            await govecClient.balance({ address: adminAddr });
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

    it("Govec should have factoryAddr as the minter", async () => {
        const m = await govecClient.minter();
        expect(m.minter).toEqual(addrs.factoryAddr);
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
