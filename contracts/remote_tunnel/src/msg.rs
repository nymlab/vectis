use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub port_id: String,
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Only factory_remote can mint govec tokens
    MintGovec {
        wallet_addr: String
    },
    // StakeGovec {}
    // UnstakeGovec {}
    // Vote
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
