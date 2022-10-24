import { InstantiateMsg as Cw20SBVInstantiateMsg } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import {
    InstantiateMsg as CwPropSingleInstantiateMsg,
    DepositInfo,
    Duration,
    Threshold,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { TokenInfo } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import { toCosmosMsg } from "@vectis/core/utils/enconding";
import CWClient from "./cosmwasm";

import { Duration as StakeDuration } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import { uploadReportPath } from "@vectis/core/utils/constants";
import { ExecuteMsg as CwPropSingleExecuteMsg, Vote } from "@dao-dao/types/contracts/cw-proposal-single";

// Proposal
//
// Cool down period for unstaking, time in seconds
// if it is not null, dao-deploy will need to wait for unstaked and claim
export const unstakeDuration: StakeDuration | null = null;
// Deposit required for creating proposal
export const depositInfo: DepositInfo | null = null;
// Length of  max Voting Period, Time in seconds
export const maxVotingPeriod: Duration | null = { time: 60 * 60 * 24 * 14 };
// Length of  min Voting Period , Time in seconds
export const minVotingPeriod: Duration | null = null;
// Can members change their votes before expiry
// It is easier for it to be false for deployment
export const allowRevote: Boolean = false;
// Details - https://docs.rs/cw-utils/0.13.2/cw_utils/enum.ThresholdResponse.html
export const threshold: Threshold = {
    threshold_quorum: {
        quorum: {
            percent: "0.6",
        },
        threshold: {
            percent: "0.3",
        },
    },
};

export function getDepositInfo(deposit: string, refundFailedProposal: boolean, govecAddr: string): DepositInfo {
    return {
        // The number of tokens that must be deposited to create a proposal.
        deposit: deposit,
        //  If failed proposals should have their deposits refunded.
        refund_failed_proposals: refundFailedProposal,
        // The address of the cw20 token to be used for proposal deposits.
        token: { token: { address: govecAddr } },
    };
}

class DaoClient {
    daoAddr: string;
    voteAddr: string;
    proposalAddr: string;
    stakingAddr: string;
    constructor(
        private readonly client: CWClient,
        { daoAddr, voteAddr, proposalAddr, stakingAddr }: Record<string, string>
    ) {
        this.daoAddr = daoAddr;
        this.voteAddr = voteAddr;
        this.proposalAddr = proposalAddr;
        this.stakingAddr = stakingAddr;
    }

    async queryProposals() {
        return await this.client.queryContractSmart(this.proposalAddr, { list_proposals: {} });
    }

    async createProposal(title: string, description: string, msgs: Record<string, unknown>[]) {
        const proposal = {
            propose: {
                description,
                latest: null,
                msgs,
                title,
            },
        };

        const res = await this.client.execute(this.client.sender, this.proposalAddr, proposal, "auto");
        console.log("\n\nProposed created\n", JSON.stringify(res));
        return res;
    }

    async voteProposal(proposalId: number, vote: Vote) {
        const msg: CwPropSingleExecuteMsg = {
            vote: {
                proposal_id: proposalId,
                vote,
            },
        };

        const res = await this.client.execute(this.client.sender, this.proposalAddr, msg, "auto");
        console.log(`\n\nVote Proposal: ${proposalId}\n`, JSON.stringify(res));
        return res;
    }

    async executeProposal(proposalId: number) {
        const msg: CwPropSingleExecuteMsg = {
            execute: {
                proposal_id: proposalId,
            },
        };
        const res = await this.client.execute(this.client.sender, this.proposalAddr, msg, "auto");
        console.log(`\n\nExecuted Proposal ${proposalId}\n`, JSON.stringify(res));
        return res;
    }

    static async instantiate(client: CWClient, govecAddr: string) {
        const { host } = await import(uploadReportPath);
        const { stakingRes, voteRes, daoRes, proposalSingleRes } = host;
        const [{ address }] = await client.getAccounts();

        const tokenInfo = DaoClient.createTokenInfo(govecAddr, stakingRes.codeId, unstakeDuration);
        const voteInstMsg = DaoClient.createVoteInstMsg(tokenInfo);
        // cw-proposal-single instantiation msg
        const propInstMsg = DaoClient.createPropInstMsg(
            depositInfo,
            maxVotingPeriod,
            minVotingPeriod,
            threshold,
            allowRevote
        );

        // dao-core instantiation msg
        // TODO: the module types `ModuleInstantiateInfo` do not work with the @daodao/types,
        // therefore not using interfaces. There is versioning issues
        // https://github.com/DA0-DA0/dao-contracts/pull/347#pullrequestreview-1011556931
        const govModInstInfo = DaoClient.createGovModInstInfo(proposalSingleRes.codeId, propInstMsg as any);
        const voteModInstInfo = DaoClient.createVoteModInstInfo(voteRes.codeId, voteInstMsg);
        const daoInstMsg = DaoClient.createDaoInstMsg(govModInstInfo, voteModInstInfo);

        console.log("dao code id: ", daoRes.codeId);

        const { contractAddress: daoAddr } = await client.instantiate(
            address,
            daoRes.codeId,
            daoInstMsg,
            "VectisDAO",
            "auto"
        );

        const voteAddr = await client.queryContractSmart(daoAddr, { voting_module: {} });
        const [proposalAddr] = await client.queryContractSmart(daoAddr, { proposal_modules: {} });
        const stakingAddr = await client.queryContractSmart(voteAddr, { staking_contract: {} });
        console.log("Instantiated DAO at: ", daoAddr);
        console.log("Instantiated Vote at: ", voteAddr);
        console.log("Instantiated proposal at: ", proposalAddr);
        console.log("Instantiated staking at: ", stakingAddr);

        return new DaoClient(client, { daoAddr, voteAddr, proposalAddr, stakingAddr });
    }

    static createGovModInstInfo(proposalCodeId: number, propInstMsg: CwPropSingleInstantiateMsg) {
        return {
            admin: {
                core_contract: {},
            },
            code_id: proposalCodeId,
            label: "Vectis Proposal Module",
            msg: toCosmosMsg<CwPropSingleInstantiateMsg>(propInstMsg),
        };
    }

    static createVoteModInstInfo(voteCodeId: number, voteInstMsg: Cw20SBVInstantiateMsg) {
        return {
            admin: {
                core_contract: {},
            },
            code_id: voteCodeId,
            label: "Vectis Vote Module",
            msg: toCosmosMsg<Cw20SBVInstantiateMsg>(voteInstMsg),
        };
    }

    static createVoteInstMsg(tokenInfo: TokenInfo) {
        return {
            token_info: tokenInfo,
        };
    }

    static createDaoInstMsg(govModInstInfo: unknown, voteModInstInfo: unknown) {
        return {
            description: "Vectis: Smart Contract Wallet",
            proposal_modules_instantiate_info: [govModInstInfo],
            name: "VectisDAO",
            voting_module_instantiate_info: voteModInstInfo,
            automatically_add_cw20s: true,
            automatically_add_cw721s: true,
        };
    }

    static createTokenInfo(tokenAddr: string, stakingCodeId: number, unstakingDuration: Duration | null): TokenInfo {
        return {
            existing: {
                address: tokenAddr,
                staking_contract: {
                    new: {
                        staking_code_id: stakingCodeId,
                        unstaking_duration: unstakingDuration,
                    },
                },
            },
        };
    }

    static createPropInstMsg(
        depositInfo: DepositInfo | null,
        maxVotingPeriod: Duration | null,
        minVotingPeriod: Duration | null,
        threshold: Threshold,
        allow_revoting: Boolean
    ) {
        return {
            // deposit required for creating proposal
            deposit_info: depositInfo,
            // time in seconds
            // {
            //     time: 60 * 60 * 24 * 14,
            // }
            max_voting_period: maxVotingPeriod,
            only_members_execute: false,
            // details - https://docs.rs/cw-utils/0.13.2/cw_utils/enum.ThresholdResponse.html
            // threshold_quorum: {
            //     quorum: {
            //         percent: "0.6",
            //     },
            //     threshold: {
            //         percent: "0.3",
            //     },
            // },
            threshold: threshold,
            /// The minimum amount of time a proposal must be open before
            /// passing. A proposal may fail before this amount of time has
            /// elapsed, but it will not pass. This can be useful for
            /// preventing governance attacks wherein an attacker aquires a
            /// large number of tokens and forces a proposal through.
            min_voting_period: minVotingPeriod,
            /// Allows changing votes before the proposal expires. If this is
            /// enabled proposals will not be able to complete early as final
            /// vote information is not known until the time of proposal
            /// expiration.
            allow_revoting: allow_revoting,
        };
    }
}

export default DaoClient;
