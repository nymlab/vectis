use crate::types::{
    plugin::{
        Plugin, PluginCodeData, PluginMetadataData, PluginWithVersionResponse, PluginsResponse,
    },
    plugin_registry::{RegistryConfigResponse, Subscriber, SubscriptionTier, TierDetails},
};
use cosmwasm_std::{Coin, Response, StdError, StdResult};
use cw2::ContractVersion;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

pub mod registry_service_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait RegistryServiceTrait {
        type Error: From<StdError>;

        /// Called by proxy contract to record increase in plugins
        /// Codehash of the caller is checked
        #[msg(exec)]
        fn proxy_install_plugin(
            &self,
            ctx: ExecCtx,
            id: u64,
            addr: String,
        ) -> Result<Response, Self::Error>;

        /// Called by proxy contract to remove the plugin in this state and in the proxy state
        /// Codehash of the caller is checked
        /// This is called with `auth_remove_plugin`
        #[msg(exec)]
        fn proxy_remove_plugins(
            &self,
            ctx: ExecCtx,
            addr: Vec<String>,
        ) -> Result<Response, Self::Error>;

        /// Called by users to subscribe to a different tier for both upgrade & downgrades
        #[msg(exec)]
        fn subscribe(&self, ctx: ExecCtx, tier: SubscriptionTier) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn subsciption_details(
            &self,
            ctx: QueryCtx,
            addr: String,
        ) -> Result<Option<Subscriber>, StdError>;
    }
}

pub mod registry_management_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait RegistryManagementTrait {
        type Error: From<StdError>;

        #[msg(exec)]
        fn register_plugin(
            &self,
            ctx: ExecCtx,
            code_data: PluginCodeData,
            metadata_data: PluginMetadataData,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn unregister_plugin(&self, ctx: ExecCtx, id: u64) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn new_plugin_version(
            &self,
            ctx: ExecCtx,
            /// The id on the vectis plugin registry
            id: u64,
            /// Code update must pump latest_contract_version
            code_update: Option<PluginCodeData>,
            /// Metadata update will not require code version pump
            metadata_update: PluginMetadataData,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_registry_fee(&self, ctx: ExecCtx, new_fee: Coin)
            -> Result<Response, Self::Error>;

        /// Adding new tiers
        /// To remove tier there may already be subscribers and so it will require a migration
        /// function
        #[msg(exec)]
        fn add_or_update_subscription_tiers(
            &self,
            ctx: ExecCtx,
            tier: u8,
            details: TierDetails,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn get_plugins(
            &self,
            ctx: QueryCtx,
            limit: Option<u32>,
            start_after: Option<u32>,
        ) -> StdResult<PluginsResponse>;

        #[msg(query)]
        fn get_plugin_by_id(&self, ctx: QueryCtx, id: u64) -> StdResult<Option<Plugin>>;

        /// This helps to do all the neccessary queries to allow caller to know the ipfs_hash
        #[msg(query)]
        fn get_plugin_by_address(
            &self,
            ctx: QueryCtx,
            contract_addr: String,
        ) -> StdResult<PluginWithVersionResponse>;

        #[msg(query)]
        fn get_config(&self, ctx: QueryCtx) -> StdResult<RegistryConfigResponse>;

        #[msg(query)]
        fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError>;
    }
}
