import { Cw3FlexT } from "../interfaces";

// Proposal for the Vectis committees multisig config
// Length of  max Voting Period, Time in seconds
export const maxVotingPeriod: Cw3FlexT.Duration = {
    time: 60 * 60 * 24 * 14,
};

// Vectis Committee Config
// Responsible for approving plugins into the Plugin registry
export const vectisCommitteeThreshold: Cw3FlexT.Threshold = {
    absolute_percentage: { percentage: "0.5" },
};
export const vectisCommittee1Weight: number = 50;
export const vectisCommittee2Weight: number = 50;
export const vectisTechCommittee1Weight: number = 50;
export const vectisTechCommittee2Weight: number = 50;

// This is manual translate onchain VectisActors to string
export enum VectisActors {
    PluginRegistry = "PluginRegistry",
    Factory = "Factory",
}
