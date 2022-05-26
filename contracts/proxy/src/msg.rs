use cosmwasm_std::{Addr, CosmosMsg, Empty};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use vectis_wallet::{CreateWalletMsg, Guardians, RelayTransaction};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub create_wallet_msg: CreateWalletMsg,
    /// Fixed Multisig Code Id for guardians
    pub multisig_code_id: u64,
    /// Code Id used to instantiate the contract
    pub code_id: u64,
    /// Chain address prefix
    pub addr_prefix: String,
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
    /// Message for updating current guardians set
    ///
    /// If the `Guardians.guardian_multisig` is given,
    /// we will instantiate a new multisig contract.
    /// This contract can be an instance of 3 code ids.
    /// - 1: exisiting stored `MULTISIG_CODE_ID` if `new_multisig_code_id == None`
    /// - 2: the `new_multisig_code_id` if given
    /// - 3: if 1 nor 2 are available, the supported multisig from the FACTORY will be used.
    UpdateGuardians {
        guardians: Guardians,
        new_multisig_code_id: Option<u64>,
    },
    /// Updates label by the user
    UpdateLabel { new_label: String },
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
