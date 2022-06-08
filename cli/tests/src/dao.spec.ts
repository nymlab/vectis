import { createSigningClient, delay } from "@vectis/core/utils/utils";
import { defaultExecuteFee, defaultInstantiateFee, walletFee } from "@vectis/core/utils/fee";
import { GovecClient } from "@vectis/types/contracts/GovecContract";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { QueryMsg as StakeQuery } from "@dao-dao/types/contracts/stake-cw20";
import { InstantiateMsg as FactoryInstantiateMsg } from "@vectis/types/contracts/FactoryContract";

import { adminAddr, addrPrefix, adminMnemonic } from "@vectis/core/utils/constants";

import { CosmosMsg_for_Empty } from "types/contracts/ProxyContract";
import { createGovModInstInfo, createTokenInfo, createVoteModInstInfo } from "./mocks/info";
import { createDaoInstMsg, createPropInstMsg, createVoteInstMsg } from "./mocks/messages";
import { toCosmosMsg } from "@vectis/core/utils/cosmwasm";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { instantiateGovec } from "@vectis/core/contracts";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */
describe("DAO Suite: ", () => {
    let adminClient: SigningCosmWasmClient;
    let govecClient: GovecClient;
    let factoryCodeId: number;
    let proxyCodeId: number;
    let multisigCodeId: number;
    let daoAddr: string;
    let stakingAddr: string;
    let propAddrs: string[];
    let proposalId: number;

    beforeAll(async () => {
        const { factoryRes, proxyRes, multisigRes, stakingRes, voteRes, govecRes, daoRes, proposalSingleRes } =
            await import("../../uploadInfo.json" as string);

        adminClient = await createSigningClient(adminMnemonic, addrPrefix);
        const { govecAddr } = await instantiateGovec(adminClient, govecRes.codeId, adminAddr);
        govecClient = new GovecClient(adminClient, adminAddr, govecAddr);

        proxyCodeId = proxyRes.codeId;
        factoryCodeId = factoryRes.codeId;
        multisigCodeId = multisigRes.codeId;

        const tokenInfo = createTokenInfo(govecAddr, stakingRes.codeId);

        const voteInstMsg = createVoteInstMsg(tokenInfo);
        // cw-proposal-single instantiation msg
        const propInstMsg = createPropInstMsg();
        // dao-core instantiation msg
        // TODO: the module types `ModuleInstantiateInfo` do not work with the @daodao/types,
        // therefore not using interfaces. There is versioning issues
        const govModInstInfo = createGovModInstInfo(proposalSingleRes.codeId, propInstMsg);
        const voteModInstInfo = createVoteModInstInfo(voteRes.codeId, voteInstMsg);
        const daoInstMsg = createDaoInstMsg(govModInstInfo, voteModInstInfo);

        const { contractAddress } = await adminClient.instantiate(
            adminAddr,
            daoRes.codeId,
            daoInstMsg,
            "VectisDAO",
            defaultInstantiateFee
        );
        daoAddr = contractAddress;
        const voteAddr = await adminClient.queryContractSmart(daoAddr, { voting_module: {} });
        propAddrs = await adminClient.queryContractSmart(daoAddr, { proposal_modules: {} });
        stakingAddr = await adminClient.queryContractSmart(voteAddr, { staking_contract: {} });
    });

    it("Should let admin set staking addr on Govec", async () => {
        await govecClient.updateStakingAddr({ newAddr: stakingAddr });
        const staking = await govecClient.staking();
        expect(stakingAddr).toBe(staking);
    });

    it("Should let admin mint for self and stake", async () => {
        await govecClient.mint({ newWallet: adminAddr });
        // todo: transfer govec admin to dao

        // admin stake to propose and vote to deploy factory contract
        const sendMsg = { stake: {} };
        await govecClient.send({ amount: "1", contract: stakingAddr, msg: toCosmosMsg(sendMsg) });

        // need this for block to be mined
        await delay(5000);

        const stakeQuery: StakeQuery = { staked_balance_at_height: { address: adminAddr! } };
        const stakingPower = await adminClient.queryContractSmart(stakingAddr, stakeQuery);
        expect(stakingPower.balance).toBe("1");
    });

    it("Should propose to deploy Factory contract as admin", async () => {
        const factorInstMsg: FactoryInstantiateMsg = {
            proxy_code_id: proxyCodeId,
            proxy_multisig_code_id: multisigCodeId,
            addr_prefix: addrPrefix!,
            wallet_fee: walletFee,
        };

        const deployFactoryMsg: CosmosMsg_for_Empty = {
            wasm: {
                instantiate: {
                    admin: daoAddr,
                    code_id: factoryCodeId,
                    funds: [],
                    label: "Vectis Factory",
                    msg: toCosmosMsg(factorInstMsg),
                },
            },
        };

        const proposalTitle = "Deploy Vectis Factory";
        const proposal: CwPropSingleExecuteMsg = {
            propose: {
                description: "Deploy Vectis Factory",
                latest: null,
                msgs: [deployFactoryMsg],
                title: "Deploy Vectis Factory",
            },
        };

        // propose
        await adminClient.execute(adminAddr!, propAddrs[0], proposal, defaultExecuteFee);
        const propQuery: ProposalQueryMsg = { list_proposals: {} };
        const props = await adminClient.queryContractSmart(propAddrs[0], propQuery);
        proposalId = props.proposals[0].id;
        expect(props.proposals[0].proposal.title).toBe(proposalTitle);
    });

    it("Should vote and execute Factory contract deployment as admin", async () => {
        const vote: CwPropSingleExecuteMsg = {
            vote: {
                proposal_id: proposalId,
                vote: "yes",
            },
        };
        await adminClient.execute(adminAddr, propAddrs[0], vote, defaultExecuteFee);

        const execute: CwPropSingleExecuteMsg = {
            execute: {
                proposal_id: proposalId,
            },
        };
        const res = await adminClient.execute(adminAddr!, propAddrs[0], execute, defaultExecuteFee);
        expect(res.logs[0].events[1]["type"]).toBe("instantiate");
        expect(res.logs[0].events[1].attributes[1].value).toBe(String(factoryCodeId));
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
