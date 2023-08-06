use cosmwasm_schema::{cw_serde, schemars, QueryResponses};
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty};

use crate::{
    CreateWalletMsg, GuardiansUpdateMsg, GuardiansUpdateRequest, PluginListResponse,
    PluginPermissions, RelayTransaction, WalletInfo,
};
use cw1::CanExecuteResponse;
use std::fmt;

#[cw_serde]
pub struct ProxyInstantiateMsg {
    pub create_wallet_msg: CreateWalletMsg,
    /// Code Id used to instantiate the contract
    pub code_id: u64,
    /// If MS guardians, the code_id, address, salt is pre-calculated can be pre-calculated on the factory
    /// There is no reply to the factory, is if this is None, MS addresses won't be on the factory
    pub ms_inst_info: Option<(u64, Addr, Binary)>,
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
    Plugins {},
}

#[cw_serde]
pub enum ProxyExecuteMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + schemars::JsonSchema,
{
    /// Execute requests the contract to re-dispatch all these messages with the
    /// contract's address as sender.
    /// This is only callable by controllers whose authentication method is support by the chain,
    /// which is only ECDSA.
    /// Priviledge: Controller
    Execute { msgs: Vec<CosmosMsg<T>> },
    /// Freeze will freeze the account in the scenario the controller lose their key / device
    /// Priviledge: Guardian/Multisig
    RevertFreezeStatus {},
    /// Relay message contains the signature and the message to relay
    /// Priviledge: Relayer
    Relay { transaction: RelayTransaction },
    /// Rotating the Controller Key
    /// Priviledge: Controller, Guardian/Multisig
    RotateControllerKey { new_controller_address: String },
    /// Adding a new relayer
    /// Priviledge: Controller/Multisig
    AddRelayer { new_relayer_address: Addr },
    /// Removing relayer
    /// Priviledge: Controller/Multisig
    RemoveRelayer { relayer_address: Addr },
    /// It create a request for update guardians and it has a delay of one day after that
    /// is possible to update the guardians using UpdateGuardiansMsg
    RequestUpdateGuardians { request: Option<GuardiansUpdateMsg> },
    /// Once the request passed the waiting time, it is possible to update the guardians.
    UpdateGuardians {},
    /// Updates label by the controller
    UpdateLabel { new_label: String },
    /// Instantiates the plugin contract
    /// Priviledge: User
    InstantiatePlugin {
        src: PluginSource,
        instantiate_msg: Binary,
        plugin_params: PluginParams,
        label: String,
    },
    /// Update plugins, if params is `None`, plugin is removed
    /// If code_id is Some, plugin is migrated
    /// If plugin_permissions is Some, plugin is added
    /// Priviledge: User
    UpdatePlugins {
        plugin_addr: String,
        plugin_permissions: Option<Vec<PluginPermissions>>,
        migrate_msg: Option<(u64, Binary)>,
    },
    /// Similar to Execute but called by plugins,
    /// this has some checks and limitations
    PluginExecute { msgs: Vec<CosmosMsg<T>> },
}

/// The source of the plugin code.
#[cw_serde]
pub enum PluginSource {
    VectisRegistry(u64),
    CodeId(u64),
}

#[cw_serde]
pub struct PluginParams {
    // Do we want to instantitate with permission for the grantor?
    // if so, this instantiate message goes directly to a grantor plugin
    pub permissions: Vec<PluginPermissions>,
}

impl PluginParams {
    pub fn has_exec_access(&self) -> bool {
        self.permissions.contains(&PluginPermissions::Exec)
    }
}
