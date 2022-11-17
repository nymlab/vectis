use cosmwasm_schema::{cw_serde, QueryResponses};

use cw1::CanExecuteResponse;

use crate::{CreateWalletMsg, GuardiansUpdateRequest, PluginListResponse, WalletInfo};

#[cw_serde]
pub struct ProxyInstantiateMsg {
    pub create_wallet_msg: CreateWalletMsg,
    /// Fixed Multisig Code Id for guardians
    pub multisig_code_id: u64,
    /// Code Id used to instantiate the contract
    pub code_id: u64,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ProxyQueryMsg {
    /// Query for wallet info
    #[returns(WalletInfo)]
    Info {},
    /// Checks permissions of the caller on this proxy.
    /// If CanExecuteRelay returns true then a call to `ExecuteRelay`,
    /// before any further state changes, should also succeed.
    #[returns(CanExecuteResponse)]
    CanExecuteRelay { sender: String },
    /// Return the current guardian update request.
    #[returns(Option<GuardiansUpdateRequest>)]
    GuardiansUpdateRequest {},
    #[returns(PluginListResponse)]
    Plugins {
        // Address string to start after
        start_after: Option<String>,
        // Max is 30 and default is 10
        limit: Option<u32>,
    },
}
