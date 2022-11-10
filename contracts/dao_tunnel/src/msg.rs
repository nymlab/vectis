use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use vectis_wallet::{DaoTunnelPacketMsg, IbcTransferChannels, Receiver};

#[cw_serde]
pub struct InstantiateMsg {
    /// Govec contract address
    pub govec_minter: String,
    /// Any remote tunnel endpoints to be included initially
    /// No channels has been established
    pub init_remote_tunnels: Option<RemoteTunnels>,
    pub init_ibc_transfer_mods: Option<IbcTransferChannels>,
    pub denom: String,
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
    UpdateDaoAddr {
        new_addr: String,
    },
    UpdateGovecAddr {
        new_addr: String,
    },
    /// The connections and channel_id this chain supports
    /// and voted in by the DAO
    UpdateIbcTransferRecieverChannel {
        connection_id: String,
        channel_id: Option<String>,
    },
    /// Transfer native tokens to another chain
    /// Fund amount is forward from the MessageInfo.funds
    IbcTransfer {
        receiver: Receiver,
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
    #[returns(Option<Addr>)]
    Dao {},
    #[returns(IbcTransferChannels)]
    IbcTransferChannels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[cw_serde]
pub struct RemoteTunnels {
    /// These are endpoints to other remote-tunnel contracts
    /// (connection_id, port_id)
    pub tunnels: Vec<(String, String)>,
}
