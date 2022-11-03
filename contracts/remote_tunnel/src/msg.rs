use cosmwasm_schema::{cw_serde, QueryResponses};
use vectis_wallet::{ChainConfig, DaoConfig, RemoteTunnelPacketMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_config: DaoConfig,
    pub chain_config: ChainConfig,
    pub denom: String,
    pub init_ibc_transfer_mod: Option<IbcTransferChannels>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Executed by proxy wallets for Dao actions
    DaoActions { msg: RemoteTunnelPacketMsg },
    /// Transfer native tokens to another chain
    /// Fund amount is forward from the MessageInfo.funds
    /// if `addr = None`, funds is transfer to the DAO
    IbcTransfer { receiver: Receiver },
}

#[cw_serde]
pub enum Receiver {
    Dao,
    Other {
        connection_id: String,
        port_id: String,
        addr: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoConfig)]
    DaoConfig {},
    #[returns(ChainConfig)]
    ChainConfig {},
    #[returns(IbcTransferChannels)]
    IbcTransferChannels {
        start_from: Option<(String, String)>,
        limit: Option<u32>,
    },
    #[returns(u64)]
    NextJobId {},
}

#[cw_serde]
pub struct IbcTransferChannels {
    /// (connection_id, port_id, channel_id)
    /// when using for instantiation, channel_id is ignored
    pub endpoints: Vec<(String, String, Option<String>)>,
}
