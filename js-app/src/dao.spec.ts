import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { uploadContracts } from "./util/contracts";
import { createSigningClient } from "./util/utils";
import { defaultExecuteFee, defaultInstantiateFee, walletFee } from "./util/fee";
import { GovecClient } from "../types/GovecContract";
import {
    InstantiateMsg as Cw20SBVInstantiateMsg,
    TokenInfo,
    QueryMsg as Cw20SBVQueryMsg,
} from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import {
    InstantiateMsg as CwPropSingleInstantiateMsg,
    CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { QueryMsg as DaoQueryMsg } from "@dao-dao/types/contracts/cw-core";
import { QueryMsg as StakeQuery } from "@dao-dao/types/contracts/stake-cw20";
import { InstantiateMsg as FactoryInstantiateMsg } from "../types/FactoryContract";
import { adminAddr, addrPrefix, adminMnemonic } from "./util/env";
import { instantiateGovec } from "./util/contracts";
import { CosmosMsg_for_Empty } from "types/ProxyContract";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */
describe("DAO Suite: ", () => {
    let adminClient: SigningCosmWasmClient;
    let govecClient: GovecClient;
    let factoryCodeId: number;
    let proxyCodeId: number;
    let multisigCodeId: number;
    let govecCodeId: number;
    let stakingCodeId: number;
    let voteCodeId: number;
    let daoCodeId: number;
    let proposalSingleCodeId: number;
    let daoAddr: string;
    let factoryAddr: string;
    let voteAddr: string;
    let stakingAddr: string;
    let propAddrs: string[];
    let goVecAddr: string;
    let proposalId: number;

    const delay = (ms: number) => {
        return new Promise((resolve) => setTimeout(resolve, ms));
    };

    beforeAll(async () => {
        adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
        const { factoryRes, proxyRes, multisigRes, govecRes, stakingRes, voteRes, daoRes, proposalSingleRes } =
            await uploadContracts(adminClient);
        factoryCodeId = factoryRes.codeId;
        proxyCodeId = proxyRes.codeId;
        multisigCodeId = multisigRes.codeId;
        govecCodeId = govecRes.codeId;
        stakingCodeId = stakingRes.codeId;
        voteCodeId = voteRes.codeId;
        daoCodeId = daoRes.codeId;
        proposalSingleCodeId = proposalSingleRes.codeId;

        // deploy Govec cw20 whitelist only contract first
        const { govecAddr } = await instantiateGovec(adminClient, govecCodeId, adminAddr!);
        goVecAddr = govecAddr;

        // cw20_staked_balance_voting instantiation msg
        const tokenInfo: TokenInfo = {
            existing: {
                address: govecAddr,
                staking_contract: {
                    new: {
                        staking_code_id: stakingCodeId,
                        unstaking_duration: { time: 60 * 60 * 1 },
                    },
                },
            },
        };

        const voteInstMsg: Cw20SBVInstantiateMsg = {
            token_info: tokenInfo,
        };

        // cw-proposal-single instantiation msg
        const propInstMsg: CwPropSingleInstantiateMsg = {
            // deposit not required for creating proposal
            deposit_info: null,
            // time in seconds
            max_voting_period: {
                time: 60 * 60 * 24 * 14,
            },
            only_members_execute: false,
            threshold: {
                // details - https://docs.rs/cw-utils/0.13.2/cw_utils/enum.ThresholdResponse.html
                threshold_quorum: {
                    quorum: {
                        percent: "0.6",
                    },
                    threshold: {
                        percent: "0.3",
                    },
                },
            },
        };

        // dao-core instantiation msg
        // TODO: the module types `ModuleInstantiateInfo` do not work with the @daodao/types,
        //       therefore not using interfaces. There is versioning issues
        const govModInstInfo = {
            admin: {
                core_contract: {},
            },
            code_id: proposalSingleCodeId,
            label: "Vectis Proposal Module",
            msg: toBase64(toUtf8(JSON.stringify(propInstMsg))),
        };

        const voteModInstInfo = {
            admin: {
                core_contract: {},
            },
            code_id: voteCodeId,
            label: "Vectis Vote Module",
            msg: toBase64(toUtf8(JSON.stringify(voteInstMsg))),
        };

        const daoInstMsg = {
            description: "Vectis: Smart Contract Wallet",
            proposal_modules_instantiate_info: [govModInstInfo],
            name: "VectisDAO",
            voting_module_instantiate_info: voteModInstInfo,
            automatically_add_cw20s: true,
            automatically_add_cw721s: true,
        };

        // deploy dao
        const { contractAddress } = await adminClient.instantiate(
            adminAddr!,
            daoCodeId,
            daoInstMsg,
            "VectisDAO",
            defaultInstantiateFee
        );
        daoAddr = contractAddress;
    });

    it("Should let admin set staking addr on Govec", async () => {
        // admin set staking addr on Govec
        const queryVote: DaoQueryMsg = { voting_module: {} };
        // TODO: incompatible @daodao/types
        const queryProp = { proposal_modules: {} };
        voteAddr = await adminClient.queryContractSmart(daoAddr, queryVote);
        propAddrs = await adminClient.queryContractSmart(daoAddr, queryProp);

        const queryStaking: Cw20SBVQueryMsg = { staking_contract: {} };
        stakingAddr = await adminClient.queryContractSmart(voteAddr, queryStaking);

        govecClient = new GovecClient(adminClient, adminAddr!, goVecAddr);
        await govecClient.updateStakingAddr({ newAddr: stakingAddr });
        const staking = await govecClient.staking();
        expect(stakingAddr).toBe(staking);
    });

    it("Should let admin mint for self and stake", async () => {
        await govecClient.mint({ newWallet: adminAddr! });
        // todo: transfer govec admin to dao

        // admin stake to propose and vote to deploy factory contract
        const sendMsg = { stake: {} };
        await govecClient.send({ amount: "1", contract: stakingAddr, msg: toBase64(toUtf8(JSON.stringify(sendMsg))) });

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
                    msg: toBase64(toUtf8(JSON.stringify(factorInstMsg))),
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
        await adminClient.execute(adminAddr!, propAddrs[0], vote, defaultExecuteFee);

        const execute: CwPropSingleExecuteMsg = {
            execute: {
                proposal_id: proposalId,
            },
        };
        const res = await adminClient.execute(adminAddr!, propAddrs[0], execute, defaultExecuteFee);
        expect(res.logs[0].events[1]["type"]).toBe("instantiate");
        expect(res.logs[0].events[1].attributes[1].value).toBe(String(factoryCodeId));
        factoryAddr = res.logs[0].events[1].attributes[0].value;
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
