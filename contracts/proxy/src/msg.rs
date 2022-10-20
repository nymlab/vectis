use cosmwasm_schema::{cw_serde, schemars, QueryResponses};
use cosmwasm_std::{Addr, CosmosMsg, Empty};
use cw1::CanExecuteResponse;
use std::fmt;
use vectis_wallet::{
    CreateWalletMsg, GuardiansUpdateMsg, GuardiansUpdateRequest, RelayTransaction, WalletInfo,
};
#[cw_serde]
pub struct InstantiateMsg {
    pub create_wallet_msg: CreateWalletMsg,
    /// Fixed Multisig Code Id for guardians
    pub multisig_code_id: u64,
    /// Code Id used to instantiate the contract
    pub code_id: u64,
    /// Chain address prefix
    pub addr_prefix: String,
}

#[cw_serde]
pub enum ExecuteMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + schemars::JsonSchema,
{
    /// Execute requests the contract to re-dispatch all these messages with the
    /// contract's address as sender.
    /// Priviledge: User
    Execute { msgs: Vec<CosmosMsg<T>> },
    /// Freeze will freeze the account in the scenario the user lose their key / device
    /// Priviledge: Guardian/Multisig
    RevertFreezeStatus {},
    /// Relay message contains the signature and the message to relay
    /// Priviledge: Relayer
    Relay { transaction: RelayTransaction },
    /// Rotating the User Key
    /// Priviledge: User, Guardian/Multisig
    RotateUserKey { new_user_address: String },
    /// Adding a new relayer
    /// Priviledge: User/Multisig
    AddRelayer { new_relayer_address: Addr },
    /// Removing relayer
    /// Priviledge: User/Multisig
    RemoveRelayer { relayer_address: Addr },
    /// It create a request for update guardians and it has a delay of one day after that
    /// is possible to update the guardians using UpdateGuardiansMsg
    RequestUpdateGuardians { request: Option<GuardiansUpdateMsg> },
    /// Once the request passed the waiting time, it is possible to update the guardians.
    UpdateGuardians {},
    /// Updates label by the user
    UpdateLabel { new_label: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
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
}
