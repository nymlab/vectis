use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use vectis_wallet::WalletFactoryInstantiateMsg;

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
    InstantiateRemoteFactory {
        /// Identifier used in the acknowledgement message
        job_id: u64,
        /// code_id of a Factory wasm code on the remote chain
        code_id: u64,
        msg: WalletFactoryInstantiateMsg,
        /// Sending channel_id, the local channel to the remote chain
        channel_id: String,
    },
    /// The Channel Id the remote tunnel use to communicate with dao-tunnel will be the
    /// channel that remote tunnel recieves the IBC package from
    UpdateRemoteTunnelChannel {
        /// Identifier used in the acknowledgement message
        job_id: u64,
        /// Sending channel_id, the local channel to the remote chain
        /// Note: NOT the channel set on the remote chain,
        /// that will be set direct on the remote-tunnel
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
