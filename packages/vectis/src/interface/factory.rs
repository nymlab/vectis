use crate::types::factory::CodeIdType;
use cosmwasm_std::{Addr, Binary, Coin, Response, StdError};
use cw2::ContractVersion;
use sylvia::types::{ExecCtx, QueryCtx};
use sylvia::{interface, schemars};

pub mod factory_service_trait {
    use crate::types::factory::{CreateWalletMsg, MigrateWalletMsg};
    use sylvia::types::ExecCtx;

    use super::*;

    /// The trait for users to interact with factory contract
    #[interface]
    pub trait FactoryServiceTrait {
        type Error: From<StdError>;

        #[msg(exec)]
        fn create_wallet(
            &self,
            ctx: ExecCtx,
            create_wallet_msg: CreateWalletMsg,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn migrate_wallet(
            &self,
            ctx: ExecCtx,
            migrations_msg: MigrateWalletMsg,
        ) -> Result<Response, Self::Error>;

        /// Returns the wallet address of this vectis ID
        #[msg(query)]
        fn wallet_by_vid(&self, ctx: QueryCtx, vid: String) -> Result<Option<Addr>, StdError>;

        /// Returns the wallet address of this vectis ID and chain_id
        #[msg(query)]
        fn wallet_by_vid_chain(
            &self,
            ctx: QueryCtx,
            vid: String,
            chain_id: String,
        ) -> Result<Option<String>, StdError>;
    }
}

pub mod factory_management_trait {

    use super::*;
    use crate::types::{
        authenticator::AuthenticatorType,
        factory::{ChainConnection, FeeType, FeesResponse},
    };

    /// The trait for deployer to interact with factory contract
    #[interface]
    pub trait FactoryManagementTrait {
        type Error: From<StdError>;

        /// Deployer only: update the newest code_id supported by Vectis
        /// removes supported ones if version is `None`;
        /// fails if code_id exist and version is `Some`;
        #[msg(exec)]
        fn update_code_id(
            &self,
            ctx: ExecCtx,
            ty: CodeIdType,
            code_id: u64,
            version: Option<String>,
            set_as_default: bool,
        ) -> Result<Response, Self::Error>;

        /// Deployer only: update the fee associated with using Vectis services
        #[msg(exec)]
        fn update_config_fee(
            &self,
            ctx: ExecCtx,
            ty: FeeType,
            new_fee: Coin,
        ) -> Result<Response, Self::Error>;

        /// Deployer only: update the supported chains
        #[msg(exec)]
        fn update_supported_interchain(
            &self,
            ctx: ExecCtx,
            chain_id: String,
            chain_connection: Option<ChainConnection>,
        ) -> Result<Response, Self::Error>;

        /// Deployer only: update address associated by the deployer role
        #[msg(exec)]
        fn update_deployer(&self, ctx: ExecCtx, addr: String) -> Result<Response, Self::Error>;

        /// Deployer only: update address associated by the wallet creator role
        #[msg(exec)]
        fn update_wallet_creator(
            &self,
            ctx: ExecCtx,
            addr: String,
        ) -> Result<Response, Self::Error>;

        /// Updates the authenticator provider
        /// if `new_code_id` and `new_inst_msg` is `None`,
        /// the `ty` assumes to exist and will be removed.
        /// Otherwise, it will be added (if does not exist) or migrated
        #[msg(exec)]
        fn update_auth_provider(
            &self,
            ctx: ExecCtx,
            ty: AuthenticatorType,
            new_code_id: Option<u64>,
            new_inst_msg: Option<Binary>,
        ) -> Result<Response, Self::Error>;

        /// Returns total wallets created
        #[msg(query)]
        fn total_created(&self, ctx: QueryCtx) -> Result<u64, StdError>;

        /// Returns existing codeIds of the proxy and others
        #[msg(query)]
        fn default_proxy_code_id(&self, ctx: QueryCtx) -> Result<u64, StdError>;

        /// Returns address of the deployer
        #[msg(query)]
        fn deployer(&self, ctx: QueryCtx) -> Result<Addr, StdError>;

        /// Returns address of the wallet creator
        #[msg(query)]
        fn wallet_creator(&self, ctx: QueryCtx) -> Result<Addr, StdError>;

        /// Returns supported chains
        #[msg(query)]
        fn supported_chains(
            &self,
            ctx: QueryCtx,
            start_after: Option<String>,
            limit: Option<u32>,
        ) -> Result<Vec<(String, ChainConnection)>, StdError>;

        /// Returns supported proxies
        #[msg(query)]
        fn supported_proxies(
            &self,
            ctx: QueryCtx,
            start_after: Option<u64>,
            limit: Option<u32>,
        ) -> Result<Vec<(u64, String)>, StdError>;

        /// Returns current fee `FeeResponse`
        #[msg(query)]
        fn fees(&self, ctx: QueryCtx) -> Result<FeesResponse, StdError>;

        /// Returns address of the authenticator
        #[msg(query)]
        fn auth_provider_addr(
            &self,
            ctx: QueryCtx,
            ty: AuthenticatorType,
        ) -> Result<Option<Addr>, StdError>;

        #[msg(query)]
        fn contract_version(&self, ctx: QueryCtx) -> Result<ContractVersion, StdError>;
    }
}
