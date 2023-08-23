use crate::types::factory::CodeIdType;
use cosmwasm_std::{Addr, Coin, Response, StdError};
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

        /// Returns the wallet address of this label
        #[msg(query)]
        fn wallet_by_label(&self, ctx: QueryCtx, label: String) -> Result<Option<Addr>, StdError>;
    }
}

pub mod factory_management_trait {

    use super::*;
    use crate::types::{
        authenticator::AuthenticatorType,
        factory::{FeeType, FeesResponse},
    };

    /// The trait for deployer to interact with factory contract
    #[interface]
    pub trait FactoryManagementTrait {
        type Error: From<StdError>;

        #[msg(exec)]
        fn update_code_id(
            &self,
            ctx: ExecCtx,
            ty: CodeIdType,
            new_code_id: u64,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_config_fee(
            &self,
            ctx: ExecCtx,
            ty: FeeType,
            new_fee: Coin,
        ) -> Result<Response, Self::Error>;

        #[msg(exec)]
        fn update_deployer(&self, ctx: ExecCtx, addr: String) -> Result<Response, Self::Error>;

        ///// Updates the authenticator provider
        ///// if `new_code_id` and `new_inst_msg` is `None`,
        ///// the `ty` assumes to exist and will be removed.
        ///// Otherwise, it will be added (if does not exist) or migrated
        //#[msg(exec)]
        //fn update_auth_provider(
        //    &self,
        //    ctx: ExecCtx,
        //    ty: AuthenticatorType,
        //    new_code_id: Option<u64>,
        //    new_inst_msg: Option<Binary>,
        //) -> Result<Response, Self::Error>;

        ///// Returns controlles / paginated
        //#[msg(query)]
        //fn controllers(
        //    &self,
        //    ctx: QueryCtx,
        //    limit: Option<u32>,
        //    start_after: Option<u32>,
        //) -> Result<Vec<Addr>, StdError>;

        /// Returns total wallets created
        #[msg(query)]
        fn total_created(&self, ctx: QueryCtx) -> Result<u64, StdError>;

        /// Returns existing codeIds of the proxy and others
        #[msg(query)]
        fn code_id(&self, ctx: QueryCtx, ty: CodeIdType) -> Result<u64, StdError>;

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
    }
}
