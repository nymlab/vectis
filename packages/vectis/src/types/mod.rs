pub mod error;
pub mod factory;
pub mod ibc;
pub mod state;
pub mod wallet;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum DaoActors {
    Govec = 0,
    Factory,
    ProposalCommittee,
    PluginCommittee,
    TreasuryCommittee,
    DaoTunnel,
    PluginRegistry,
    Staking,
    PreProposalModule,
}

impl std::fmt::Display for DaoActors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
