use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::CosmosMsg;

use vectis_wallet::WalletFactoryInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    AddApprovedController {
        connection_id: String,
        port_id: String,
    },
    InstantiateRemoteFactory {
        code_id: u64,
        msg: WalletFactoryInstantiateMsg,
        channel_id: String,
    },
    Dispatch {
        msgs: Vec<CosmosMsg>,
        job_id: Option<String>,
        channel_id: String,
    },
    UpdateRemoteTunnelChannel {
        channel_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
