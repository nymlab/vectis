use crate::types::wallet::WalletInfo;
use cosmwasm_std::{Binary, CosmosMsg, Response, StdError};
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

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

        /// Returns the data given the key
        #[msg(query)]
        fn data(&self, ctx: QueryCtx, key: Binary) -> Result<Option<Binary>, StdError>;

        #[msg(exec)]
        fn auth_exec(
            &self,
            ctx: ExecCtx,
            transaction: RelayTransaction,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_data(
            &self,
            ctx: ExecCtx,
            //  TODO: Would be great to use `Record` but might have ts-codegen error
            data: Vec<(Binary, Option<Binary>)>,
        ) -> Result<Response, Self::Error>;
    }
}

pub mod wallet_plugin_trait {
    use crate::types::plugin::{PluginInstallParams, PluginListResponse, PluginMigrateParams};

    use super::*;

    /// The trait for each authenticator contract
    #[interface]
    pub trait WalletPluginTrait {
        type Error: From<StdError>;

        /// Returns all installed plugins by types
        #[msg(query)]
        fn plugins(&self, ctx: QueryCtx) -> Result<PluginListResponse, StdError>;

        #[msg(exec)]
        fn plugin_execute(
            &self,
            ctx: ExecCtx,
            msg: Vec<CosmosMsg>,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn install_plugins(
            &self,
            ctx: ExecCtx,
            install: Vec<PluginInstallParams>,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_plugins(
            &self,
            ctx: ExecCtx,
            migrate: Vec<PluginMigrateParams>,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn remove_plugins(
            &self,
            ctx: ExecCtx,
            plugin_addrs: Vec<String>,
        ) -> Result<Response, Self::Error>;
    }
}