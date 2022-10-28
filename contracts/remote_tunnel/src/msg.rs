use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use vectis_wallet::RemoteTunnelPacketMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_tunnel_port_id: String,
    pub ibc_transfer_port_id: String,
    pub connection_id: String,
    pub dao_addr: String,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    DaoActions {
        msg: RemoteTunnelPacketMsg,
    },
    /// Transfer funds to the dao-chain
    /// if `addr = None`, funds is transfer to the DAO
    /// Fund amount is forward from the WasmMsg.funds
    IbcTransfer {
        addr: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<Addr>)]
    Factory {},
    #[returns(Option<String>)]
    Channel {},
}
