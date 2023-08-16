use crate::types::wallet::WalletInfo;
use cosmwasm_std::{Binary, CosmosMsg, Response, StdError};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

// We have to put traits into mods for Sylvia
pub mod wallet_trait {
    use super::*;
    use crate::types::wallet::RelayTransaction;

    /// The trait for each authenticator contract
    #[interface]
    pub trait WalletTrait {
        type Error: From<StdError>;

        /// Returns the wallet info
        #[msg(query)]
        fn info(&self, ctx: QueryCtx) -> Result<WalletInfo, StdError>;

        #[msg(exec)]
        fn auth_exec(
            &self,
            ctx: ExecCtx,
            transaction: RelayTransaction,
        ) -> Result<Response, Self::Error>;
    }
}

pub mod wallet_plugin_trait {
    use crate::types::wallet::{PluginListResponse, PluginParams, PluginPermissions, PluginSource};

    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait WalletPluginTrait {
        type Error: From<StdError>;

        /// Returns plugins by types
        #[msg(query)]
        fn plugins(&self, ctx: QueryCtx) -> Result<PluginListResponse, StdError>;

        #[msg(exec)]
        fn plugin_execute(
            &self,
            ctx: ExecCtx,
            msg: Vec<CosmosMsg>,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn install_plugin(
            &self,
            ctx: ExecCtx,
            src: PluginSource,
            instantiate_msg: Binary,
            pulgin_params: PluginParams,
            label: String,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_plugin(
            &self,
            ctx: ExecCtx,
            plugin_addr: String,
            plugin_permissions: Option<Vec<PluginPermissions>>,
            migrate_msg: Option<(u64, Binary)>,
        ) -> Result<Response, Self::Error>;
    }
}
