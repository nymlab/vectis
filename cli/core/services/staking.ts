import { Duration, TokenInfo } from "@dao-dao/types/contracts/cw20-staked-balance-voting";

export const createTokenInfo = (
    tokenAddr: string,
    stakingCodeId: number,
    unstakingDuration: Duration | null
): TokenInfo => {
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
};
