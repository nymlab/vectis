import { uploadReportPath } from "../utils/constants";
import { toCosmosMsg } from "../utils/enconding";

import CWClient from "./cosmwasm";

import type {
    InstantiateMsg as CwPrePropSingleInstantiateMsg,
    ExecuteMsg as CwPrePropSingleExecuteMsg,
    UncheckedDepositInfo,
    DepositToken,
} from "../interfaces/DaoPreProposeApprovalSingel.types";

import type { Threshold as Cw3Threshold } from "../interfaces/Cw3Flex.types";

import type {
    InstantiateMsg as CwPropSingleInstantiateMsg,
    ExecuteMsg as CwPropSingleExecuteMsg,
    PreProposeInfo,
} from "@dao-dao/types/contracts/DaoProposalSingle.v2";
import { Vote, Threshold } from "@dao-dao/types/contracts/DaoProposalSingle.common";

import type { DepositRefundPolicy } from "@dao-dao/types/contracts/common";
import type { Duration } from "@dao-dao/types/contracts/common";
import type {
    InstantiateMsg as Cw20SBVInstantiateMsg,
    TokenInfo,
    ActiveThreshold,
} from "@dao-dao/types/contracts/DaoVotingCw20Staked";
import { ModuleInstantiateInfo } from "@dao-dao/types/contracts";

// Proposal config
export const closeProposalOnExecutionFailure: boolean = false;
export const onlyMemberExecute: boolean = false;
// Length of  max Voting Period, Time in seconds
export const maxVotingPeriod: Duration = {
    time: 60 * 60 * 24 * 14,
};
// Length of  min Voting Period , Time in seconds
export const minVotingPeriod: Duration | null = null;
// Can members change their votes before expiry
// It is easier for it to be false for deployment
export const allowRevote: boolean = false;

// Duration for unstake
export const unstakingDuration: Duration = { time: 6 * 60 * 24 };
export const activeThreshold: ActiveThreshold | null = null;

export const proposalDepositAmount = "2";
/// For Pre Proposal which handles the deposit now
export const depositInfo: UncheckedDepositInfo = {
    amount: proposalDepositAmount,
    denom: { voting_module_token: {} },
    refund_policy: "always" as DepositRefundPolicy,
};
// True if anyone can submit pre-proposals
// preproposals are vetted by the proposal committee
const openPropSub: boolean = true;

// Dao proposal Threshold. Details -
// https://docs.rs/cw-utils/0.13.2/cw_utils/enum.ThresholdResponse.html
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

// PreProposal Config
// Length of  max Voting Period, Time in seconds
export const preProMaxVotingPeriod: Duration = {
    time: 60 * 60 * 24 * 14,
};
export const prePropThreshold: Cw3Threshold = {
    absolute_percentage: { percentage: "0.5" },
};
export const preProposalCommitte1Weight: number = 50;
export const preProposalCommitte2Weight: number = 50;

// Technical Committee Config
// Responsible for approving plugins into the Plugin registry
export const technicalCommittee: Cw3Threshold = {
    absolute_percentage: { percentage: "0.5" },
};
export const technicalCommittee1Weight: number = 50;
export const technicalCommittee2Weight: number = 50;

class DaoClient {
    daoAddr: string;
    voteAddr: string;
    proposalAddr: string;
    preProposalAddr: string;
    stakingAddr: string;
    constructor(
        private readonly client: CWClient,
        { daoAddr, voteAddr, preProposalAddr, proposalAddr, stakingAddr }: Record<string, string>
    ) {
        this.daoAddr = daoAddr;
        this.voteAddr = voteAddr;
        this.proposalAddr = proposalAddr;
        this.preProposalAddr = preProposalAddr;
        this.stakingAddr = stakingAddr;
    }

    async item(key: string) {
        return await this.client.queryContractSmart(this.daoAddr, {
            get_item: { key },
        });
    }

    async queryProposals() {
        return await this.client.queryContractSmart(this.proposalAddr, {
            list_proposals: {},
        });
    }

    async queryAdmin() {
        return await this.client.queryContractSmart(this.daoAddr, {
            admin: {},
        });
    }

    async querySetItems() {
        return await this.client.queryContractSmart(this.daoAddr, {
            list_items: {},
        });
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

    async executeAdminMsg(msg: unknown) {
        const dao_msg = {
            execute_admin_msgs: {
                msgs: [msg],
            },
        };
        const res = await this.client.execute(this.client.sender, this.daoAddr, dao_msg, "auto");
        return res;
    }

    async executeRemoveAdmin() {
        const dao_msg = {
            nominate_admin: {
                admin: null,
            },
        };
        const res = await this.client.execute(this.client.sender, this.daoAddr, dao_msg, "auto");
        console.log(`\n\nRemove Admin Messages \n`, JSON.stringify(res));
        return res;
    }

    static async instantiate(client: CWClient, govecAddr: string, daoAdmin: string | null, prePropApprover: string) {
        const { host } = await import(uploadReportPath);
        const { stakingRes, voteRes, daoRes, proposalSingleRes, preProposalSingleRes } = host;

        const prePropInstMsg: CwPrePropSingleInstantiateMsg = DaoClient.createPrePropInstMsg(
            depositInfo,
            openPropSub,
            prePropApprover
        );
        // cw-proposal-single instantiation msg
        const preProposalInfo: PreProposeInfo = {
            module_may_propose: {
                info: {
                    admin: {
                        core_module: {},
                    },
                    code_id: preProposalSingleRes.codeId,
                    label: "Vectis Pre Prop Module",
                    msg: toCosmosMsg<CwPrePropSingleInstantiateMsg>(prePropInstMsg),
                },
            },
        };
        const propInstMsg: CwPropSingleInstantiateMsg = DaoClient.createPropInstMsg(
            maxVotingPeriod,
            minVotingPeriod,
            threshold,
            allowRevote,
            closeProposalOnExecutionFailure,
            preProposalInfo,
            onlyMemberExecute
        );

        // dao-core instantiation msg
        // TODO: the module types `ModuleInstantiateInfo` do not work with the
        // @daodao/types, therefore not using interfaces. There is versioning
        // issues
        // https://github.com/DA0-DA0/dao-contracts/pull/347#pullrequestreview-1011556931
        const propModInstInfo: ModuleInstantiateInfo = DaoClient.createGovModInstInfo(
            proposalSingleRes.codeId,
            propInstMsg as any
        );

        const tokenInfo: TokenInfo = {
            existing: {
                address: govecAddr,
                staking_contract: {
                    new: {
                        staking_code_id: stakingRes.codeId,
                        unstaking_duration: unstakingDuration,
                    },
                },
            },
        };
        const voteInstMsg: Cw20SBVInstantiateMsg = {
            active_threshold: activeThreshold,
            token_info: tokenInfo,
        };

        const voteModInstInfo: ModuleInstantiateInfo = DaoClient.createVoteModInstInfo(voteRes.codeId, voteInstMsg);
        const daoInstMsg = DaoClient.createDaoInstMsg(propModInstInfo, voteModInstInfo, daoAdmin);

        const { contractAddress: daoAddr } = await client.instantiate(
            client.sender,
            daoRes.codeId,
            daoInstMsg,
            "VectisDAO",
            "auto"
        );

        const voteAddr = await client.queryContractSmart(daoAddr, {
            voting_module: {},
        });
        const proposals = await client.queryContractSmart(daoAddr, {
            proposal_modules: {},
        });
        const stakingAddr = await client.queryContractSmart(voteAddr, {
            staking_contract: {},
        });

        console.log("Instantiated DAO at: ", daoAddr);
        console.log("Instantiated Vote at: ", voteAddr);
        console.log("Instantiated proposal at: ", proposals[0].address);
        console.log("Instantiated staking at: ", stakingAddr);
        const policy = await client.queryContractSmart(proposals[0].address, {
            proposal_creation_policy: {},
        });
        console.log("policy: ", policy);
        console.log("Instantiated preproposal at: ", policy.module.addr);
        const preProposalAddr = policy.module.addr;
        return new DaoClient(client, {
            daoAddr,
            voteAddr,
            proposalAddr: proposals[0].address,
            preProposalAddr,
            stakingAddr,
        });
    }

    static createGovModInstInfo(
        proposalCodeId: number,
        propInstMsg: CwPropSingleInstantiateMsg
    ): ModuleInstantiateInfo {
        return {
            admin: {
                core_module: {},
            },
            code_id: proposalCodeId,
            label: "Vectis Proposal Module",
            msg: toCosmosMsg<CwPropSingleInstantiateMsg>(propInstMsg),
        };
    }

    static createVoteModInstInfo(voteCodeId: number, voteInstMsg: Cw20SBVInstantiateMsg) {
        return {
            admin: {
                core_module: {},
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

    static createDaoInstMsg(govModInstInfo: unknown, voteModInstInfo: unknown, admin: string | null) {
        return {
            admin: admin,
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
        maxVotingPeriod: Duration,
        minVotingPeriod: Duration | null,
        threshold: Threshold,
        allow_revoting: boolean,
        close_proposal_on_execution_failure: boolean,
        preProposalInfo: PreProposeInfo,
        only_members_execute: boolean
    ) {
        return {
            close_proposal_on_execution_failure: close_proposal_on_execution_failure,

            pre_propose_info: preProposalInfo,
            // time in seconds
            // {
            //     time: 60 * 60 * 24 * 14,
            // }
            max_voting_period: maxVotingPeriod,
            only_members_execute: only_members_execute,
            // details -
            // https://docs.rs/cw-utils/0.13.2/cw_utils/enum.ThresholdResponse.html
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

    static createPrePropInstMsg(
        depositInfo: UncheckedDepositInfo | null,
        open_proposal_submission: boolean,
        approver: string
    ) {
        return {
            deposit_info: depositInfo,
            extension: { approver: approver },
            open_proposal_submission: open_proposal_submission,
        };
    }

    static addApprovedControllerMsg(daoTunnelAddr: string, connectId: string, portId: string) {
        return {
            wasm: {
                execute: {
                    contract_addr: daoTunnelAddr,
                    funds: [],
                    msg: toCosmosMsg({
                        add_approved_controller: {
                            connection_id: connectId,
                            port_id: portId,
                        },
                    }),
                },
            },
        };
    }

    static removeApprovedControllerMsg(daoTunnelAddr: string, connectId: string, portId: string) {
        return {
            wasm: {
                execute: {
                    contract_addr: daoTunnelAddr,
                    funds: [],
                    msg: toCosmosMsg({
                        remove_approved_controller: {
                            connection_id: connectId,
                            port_id: portId,
                        },
                    }),
                },
            },
        };
    }

    static executeMsg(contractAddr: string, msg: unknown) {
        return {
            wasm: {
                execute: {
                    contract_addr: contractAddr,
                    funds: [],
                    msg: toCosmosMsg(msg),
                },
            },
        };
    }

    addApprovedControllerMsg(daoTunnelAddr: string, connectId: string, portId: string) {
        return DaoClient.addApprovedControllerMsg(daoTunnelAddr, connectId, portId);
    }
    removeApprovedControllerMsg(daoTunnelAddr: string, connectId: string, portId: string) {
        return DaoClient.removeApprovedControllerMsg(daoTunnelAddr, connectId, portId);
    }

    executeMsg(contractAddr: string, msg: unknown) {
        return DaoClient.executeMsg(contractAddr, msg);
    }
}

export default DaoClient;
