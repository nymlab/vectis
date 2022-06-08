import { TokenInfo } from "@dao-dao/types/contracts/cw20-staked-balance-voting";

export const createTokenInfo = (adminAddr: string, stakingCodeId: number): TokenInfo => {
    return {
        existing: {
            address: adminAddr,
            staking_contract: {
                new: {
                    staking_code_id: stakingCodeId,
                    unstaking_duration: { time: 60 * 60 * 1 },
                },
            },
        },
    };
};
