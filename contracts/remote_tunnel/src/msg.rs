use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
use vectis_wallet::{ChainConfig, DaoConfig};

pub use vectis_wallet::{IbcTransferChannels, Receiver, RemoteTunnelExecuteMsg as ExecuteMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub dao_config: DaoConfig,
    pub chain_config: ChainConfig,
    pub init_ibc_transfer_mod: Option<IbcTransferChannels>,
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
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(u64)]
    NextJobId {},
}

#[cw_serde]
pub struct ChainConfigResponse {
    pub remote_factory: Option<Addr>,
    pub denom: String,
}
