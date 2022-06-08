import {
    TokenInfo,
    InstantiateMsg as Cw20SBVInstantiateMsg,
} from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import { toCosmosMsg } from "@vectis/core/utils/cosmwasm";
import { InstantiateMsg as CwPropSingleInstantiateMsg } from "@dao-dao/types/contracts/cw-proposal-single";

export const createTokenInfo = (govecAddr: string, stakingCodeId: number): TokenInfo => {
    return {
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
};

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
