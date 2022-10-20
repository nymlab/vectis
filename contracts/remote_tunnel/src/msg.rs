use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, CosmosMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub port_id: String,
    pub connection_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintGovec {
        wallet_addr: String,
    },
    Dispatch {
        msgs: Vec<CosmosMsg>,
        job_id: Option<String>,
    },
    IbcTransfer {
        amount: Coin,
        to_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<Addr>)]
    Factory,
    #[returns(Option<String>)]
    Channel,
}
