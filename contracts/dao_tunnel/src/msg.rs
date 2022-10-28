use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use vectis_wallet::WalletFactoryInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub govec_minter: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Adds approved controller contract,
    /// i.e. remote tunnels
    AddApprovedController {
        connection_id: String,
        port_id: String,
    },
    InstantiateRemoteFactory {
        code_id: u64,
        msg: WalletFactoryInstantiateMsg,
        channel_id: String,
    },
    UpdateRemoteTunnelChannel {
        channel_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<u64>)]
    Controllers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Option<Addr>)]
    Govec {},
}

#[cw_serde]
pub struct RemoteTunnels {
    pub tunnels: Vec<(String, String)>,
}
