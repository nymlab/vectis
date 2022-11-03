use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use vectis_wallet::DaoTunnelPacketMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub govec_minter: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Adds approved ibc controller contract,
    /// i.e. remote tunnels
    AddApprovedController {
        /// The remote chain's light client identifier
        connection_id: String,
        /// The port of the remote-tunnel in the IbcChannel endpoint
        port_id: String,
    },
    RemoveApprovedController {
        /// The remote chain's light client identifier
        connection_id: String,
        /// The port of the remote-tunnel in the IbcChannel endpoint
        port_id: String,
    },
    DispatchActionOnRemoteTunnel {
        /// Identifier used in the acknowledgement message
        job_id: u64,
        msg: DaoTunnelPacketMsg,
        /// Sending channel_id, the local channel to the remote chain
        channel_id: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RemoteTunnels)]
    Controllers {
        // starts after the given connection_id and port_id
        start_after: Option<(String, String)>,
        limit: Option<u32>,
    },
    #[returns(Option<Addr>)]
    Govec {},
}

#[cw_serde]
pub struct RemoteTunnels {
    pub tunnels: Vec<(String, String)>,
}
