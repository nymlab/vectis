use cosmwasm_schema::{cw_serde, QueryResponses};
use vectis_wallet::DaoConfig;

pub use vectis_wallet::{IbcTransferChannels, Receiver, RemoteTunnelExecuteMsg as ExecuteMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_config: DaoConfig,
    pub init_ibc_transfer_mod: Option<IbcTransferChannels>,
    pub init_items: Option<Vec<(String, String)>>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(DaoConfig)]
    DaoConfig {},
    #[returns(String)]
    GetItem { key: String },
    #[returns(IbcTransferChannels)]
    IbcTransferChannels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(u64)]
    NextJobId {},
}
