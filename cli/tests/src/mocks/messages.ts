import {
    InstantiateMsg as Cw20SBVInstantiateMsg,
    QueryMsg as Cw20SBVQueryMsg,
    TokenInfo,
} from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import {
    InstantiateMsg as CwPropSingleInstantiateMsg,
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";

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

export const createPropInstMsg = (): CwPropSingleInstantiateMsg => {
    return {
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
};
