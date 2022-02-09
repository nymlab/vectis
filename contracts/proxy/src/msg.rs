use cosmwasm_std::{Addr, CosmosMsg, Empty};
use sc_wallet::{CreateWalletMsg, RelayTransaction};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub create_wallet_msg: CreateWalletMsg,
    /// Fixed Multisig Code Id for guardians
    pub multisig_code_id: u64,
    /// Factory address
    pub factory: Addr,
    /// Code Id used to instantiate the contract
    pub code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Execute requests the contract to re-dispatch all these messages with the
    /// contract's address as sender.
    /// Priviledge: User
    Execute {
        msgs: Vec<CosmosMsg<T>>,
    },
    /// Freeze will freeze the account in the scenario the user lose their key / device
    /// Priviledge: Guardian/Multisig
    RevertFreezeStatus {},
    /// Relay message contains the signature and the message to relay
    /// Priviledge: Relayer
    Relay {
        transaction: RelayTransaction,
    },
    // /// Rotating the User Key
    // /// Priviledge: Guardian/Multisig
    RotateUserKey {
        new_user_address: String,
    },
    // /// Adding a new relayer
    // /// Priviledge: User/Multisig
    AddRelayer {
        new_relayer_address: Addr,
    },
    // /// Removing relayer
    // /// Priviledge: User/Multisig
    RemoveRelayer {
        relayer_address: Addr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Query for wallet info
    Info {},
    /// Checks permissions of the caller on this proxy.
    /// If CanExecuteRelay returns true then a call to `ExecuteRelay`,
    /// before any further state changes, should also succeed.
    CanExecuteRelay { sender: String },
}
