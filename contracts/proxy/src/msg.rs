use cosmwasm_schema::{cw_serde, schemars};
use cosmwasm_std::{Addr, CosmosMsg, Empty};

use std::fmt;
use vectis_wallet::{GuardiansUpdateMsg, RelayTransaction};
pub use vectis_wallet::{ProxyInstantiateMsg as InstantiateMsg, ProxyQueryMsg as QueryMsg};

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
