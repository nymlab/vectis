use crate::types::plugin::{
    ConfigResponse, Fees, Plugin, PluginWithVersionResponse, PluginsResponse,
};
use cosmwasm_std::{Binary, Coin, Response, StdError, StdResult};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

// We have to put traits into mods for Sylvia
pub mod registry_trait {
    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait RegistryTrait {
        type Error: From<StdError>;

        #[msg(exec)]
        fn proxy_install_plugin(
            &self,
            ctx: ExecCtx,
            id: u64,
            instantiate_msg: Binary,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn register_plugin(
            &self,
            ctx: ExecCtx,
            // This needs to be the same name as `CONTRACT_NAME`
            // because when we do queries, we are only given the contract address,
            // the contract info is what is set in cw2
            name: String,
            creator: String,
            ipfs_hash: String,
            version: String,
            code_id: u64,
            checksum: String,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn unregister_plugin(&self, ctx: ExecCtx, id: u64) -> Result<Response, Self::Error>;

        #[msg(exec)]
        #[allow(clippy::too_many_arguments)]
        fn update_plugin(
            &self,
            ctx: ExecCtx,
            id: u64,
            creator: Option<String>,
            version: String,
            ipfs_hash: Option<String>,
            code_id: Option<u64>,
            checksum: Option<String>,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_registry_fee(&self, ctx: ExecCtx, new_fee: Coin)
            -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_install_fee(&self, ctx: ExecCtx, new_fee: Coin) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_deployer_addr(
            &self,
            ctx: ExecCtx,
            new_addr: String,
        ) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn get_config(&self, ctx: QueryCtx) -> StdResult<ConfigResponse>;

        #[msg(query)]
        fn get_plugins(
            &self,
            ctx: QueryCtx,
            limit: Option<u32>,
            start_after: Option<u32>,
        ) -> StdResult<PluginsResponse>;

        #[msg(query)]
        fn get_plugin_by_id(&self, ctx: QueryCtx, id: u64) -> StdResult<Option<Plugin>>;

        #[msg(query)]
        fn get_fees(&self, ctx: QueryCtx) -> StdResult<Fees>;

        /// This helps to do all the neccessary queries to allow called to know the ipfs_hash
        #[msg(query)]
        fn query_plugin_by_address(
            &self,
            ctx: QueryCtx,
            contract_addr: String,
        ) -> StdResult<PluginWithVersionResponse>;
    }
}
