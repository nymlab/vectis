use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use vectis_wallet::{ChainConfig, DaoConfig, RemoteTunnelPacketMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_config: DaoConfig,
    pub chain_config: ChainConfig,
    pub init_ibc_transfer_mod: Option<IbcTransferChannels>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Executed by proxy wallets for Dao actions
    DaoActions { msg: RemoteTunnelPacketMsg },
    /// Transfer native tokens to another chain
    /// Fund amount is forward from the MessageInfo.funds
    IbcTransfer { receiver: Receiver },
}

#[cw_serde]
pub struct Receiver {
    pub connection_id: String,
    pub addr: String,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoConfig)]
    DaoConfig {},
    #[returns(ChainConfigResponse)]
    ChainConfig {},
    #[returns(IbcTransferChannels)]
    IbcTransferChannels {
        start_from: Option<String>,
        limit: Option<u32>,
    },
    #[returns(u64)]
    NextJobId {},
}

#[cw_serde]
pub struct IbcTransferChannels {
    /// (connection_id, channel_id)
    /// The channel_id are for channel already established
    pub endpoints: Vec<(String, String)>,
}

#[cw_serde]
pub struct ChainConfigResponse {
    pub remote_factory: Option<Addr>,
    pub denom: String,
}
