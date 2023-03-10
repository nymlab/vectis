pub use crate::common::common::*;
pub use crate::common::dao_common::*;

/// PluginsSuite
///
/// PluginsSuite is a suite of contracts that are used to test the plugin registry
/// - registry

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PluginsSuite {
    #[derivative(Debug = "ignore")]
    // proxy address
    pub proxy: Addr,
    // dao test suite
    pub ds: DaoChainSuite,
}

impl PluginsSuite {
    pub fn init() -> Result<PluginsSuite> {
        let mut dao_chain_suite = crate::common::dao_common::DaoChainSuite::init().unwrap();
        let proxy = dao_chain_suite
            .create_new_proxy(dao_chain_suite.controller.clone(), vec![], None, WALLET_FEE)
            .unwrap();

        Ok(PluginsSuite {
            ds: dao_chain_suite,
            proxy,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_plugin(
        &mut self,
        sender: &Addr,
        code_id: u64,
        name: String,
        ipfs_hash: String,
        creator: String,
        checksum: String,
        version: String,
        funds: &[Coin],
    ) -> Result<(), PRegistryContractError> {
        self.ds
            .app
            .execute_contract(
                sender.clone(),
                self.ds.plugin_registry.clone(),
                &PRegistryExecMsg::RegisterPlugin {
                    code_id,
                    name,
                    ipfs_hash,
                    creator,
                    checksum,
                    version,
                },
                funds,
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn register_plugin_mocked(
        &mut self,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), PRegistryContractError> {
        self.register_plugin(
            sender,
            1,
            "plugin_1".to_string(),
            "ipfs_hash".to_string(),
            self.ds.deployer.to_string(),
            "checksum".to_string(),
            "0.0.1".to_string(),
            funds,
        )
    }

    pub fn unregister_plugin(
        &mut self,
        sender: &Addr,
        id: u64,
    ) -> Result<(), PRegistryContractError> {
        self.ds
            .app
            .execute_contract(
                sender.clone(),
                self.ds.plugin_registry.clone(),
                &PRegistryExecMsg::UnregisterPlugin { id },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_plugin(
        &mut self,
        sender: &Addr,
        id: u64,
        code_id: Option<u64>,
        name: Option<String>,
        creator: Option<String>,
        ipfs_hash: Option<String>,
        checksum: Option<String>,
        version: Option<String>,
    ) -> Result<(), PRegistryContractError> {
        self.ds
            .app
            .execute_contract(
                sender.clone(),
                self.ds.plugin_registry.clone(),
                &PRegistryExecMsg::UpdatePlugin {
                    id,
                    code_id,
                    creator,
                    name,
                    ipfs_hash,
                    checksum,
                    version,
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn update_registry_fee(
        &mut self,
        sender: &Addr,
        new_fee: Coin,
    ) -> Result<(), PRegistryContractError> {
        self.ds
            .app
            .execute_contract(
                sender.clone(),
                self.ds.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateRegistryFee { new_fee },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn update_dao_addr(
        &mut self,
        sender: &Addr,
        new_addr: &str,
    ) -> Result<(), PRegistryContractError> {
        self.ds
            .app
            .execute_contract(
                sender.clone(),
                self.ds.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateDaoAddr {
                    new_addr: new_addr.to_string(),
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn query_plugin(&self, id: u64) -> StdResult<Option<Plugin>> {
        let msg = PRegistryQueryMsg::GetPluginById { id };
        self.ds
            .app
            .wrap()
            .query_wasm_smart(self.ds.plugin_registry.clone(), &msg)
    }

    pub fn query_config(&self) -> StdResult<ConfigResponse> {
        let msg = PRegistryQueryMsg::GetConfig {};
        self.ds
            .app
            .wrap()
            .query_wasm_smart(self.ds.plugin_registry.clone(), &msg)
    }

    pub fn query_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<u32>,
    ) -> StdResult<PluginsResponse> {
        let msg = PRegistryQueryMsg::GetPlugins { limit, start_after };
        self.ds
            .app
            .wrap()
            .query_wasm_smart(self.ds.plugin_registry.clone(), &msg)
    }

    pub fn query_installed_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<String>,
    ) -> StdResult<PluginListResponse> {
        let msg = ProxyQueryMsg::Plugins { start_after, limit };
        self.ds
            .app
            .wrap()
            .query_wasm_smart(self.proxy.clone(), &msg)
    }
}
