import { InstantiateMsg as Cw20SBVInstantiateMsg } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import {
    InstantiateMsg as CwPropSingleInstantiateMsg,
    DepositInfo,
    Duration,
    Threshold,
} from "@dao-dao/types/contracts/cw-proposal-single";
import { TokenInfo } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import { toCosmosMsg } from "@vectis/core/utils/enconding";

export const createGovModInstInfo = (proposalCodeId: number, propInstMsg: CwPropSingleInstantiateMsg) => {
    return {
        admin: {
            core_contract: {},
        },
        code_id: proposalCodeId,
        label: "Vectis Proposal Module",
        msg: toCosmosMsg<CwPropSingleInstantiateMsg>(propInstMsg),
    };
};

export const createVoteModInstInfo = (voteCodeId: number, voteInstMsg: Cw20SBVInstantiateMsg) => {
    return {
        admin: {
            core_contract: {},
        },
        code_id: voteCodeId,
        label: "Vectis Vote Module",
        msg: toCosmosMsg<Cw20SBVInstantiateMsg>(voteInstMsg),
    };
};

export const createVoteInstMsg = (tokenInfo: TokenInfo): Cw20SBVInstantiateMsg => {
    return {
        token_info: tokenInfo,
    };
};

export const createDaoInstMsg = (govModInstInfo: unknown, voteModInstInfo: unknown) => {
    return {
        description: "Vectis: Smart Contract Wallet",
        proposal_modules_instantiate_info: [govModInstInfo],
        name: "VectisDAO",
        voting_module_instantiate_info: voteModInstInfo,
        automatically_add_cw20s: true,
        automatically_add_cw721s: true,
    };
};

export const createPropInstMsg = (
    depositInfo: DepositInfo | null,
    maxVotingPeriod: Duration,
    threshold: Threshold
): CwPropSingleInstantiateMsg => {
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
    };
};
