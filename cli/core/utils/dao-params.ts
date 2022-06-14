import { coinMinDenom } from "./constants";
import { Coin } from "@vectis/types/contracts/FactoryContract";
import { Duration as StakeDuration } from "@dao-dao/types/contracts/cw20-staked-balance-voting";
import { DepositInfo, Duration, Threshold } from "@dao-dao/types/contracts/cw-proposal-single";

// Next verion: Minter Cap (does Factory have a mint cap for Govec?)
// Fee for wallet creation
export const walletFee: Coin = { amount: "10000", denom: coinMinDenom! };
// Cool down period for unstaking, time in seconds
// if it is not null, dao-deploy will need to wait for unstaked and claim
export const unstakeDuration: StakeDuration | null = null;
// Deposit required for creating proposal
export const depositInfo: DepositInfo | null = null;
// Length of  , Time in seconds
export const maxVotingPeriod: Duration = { time: 60 * 60 * 24 * 14 };
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
