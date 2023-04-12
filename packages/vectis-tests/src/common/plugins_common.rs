pub use crate::common::base_common::*;
pub use crate::common::common::*;

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
    // hub test suite
    pub hub: HubChainSuite,
}

impl PluginsSuite {
    pub fn init() -> Result<PluginsSuite> {
        let mut hub_chain_suite = HubChainSuite::init().unwrap();
        let proxy = hub_chain_suite
            .create_new_proxy(hub_chain_suite.controller.clone(), vec![], None, WALLET_FEE)
            .unwrap();

        Ok(PluginsSuite {
            hub: hub_chain_suite,
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
        self.hub
            .app
            .execute_contract(
                sender.clone(),
                self.hub.plugin_registry.clone(),
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
            self.hub.deployer.to_string(),
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
        self.hub
            .app
            .execute_contract(
                sender.clone(),
                self.hub.plugin_registry.clone(),
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
        self.hub
            .app
            .execute_contract(
                sender.clone(),
                self.hub.plugin_registry.clone(),
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
        self.hub
            .app
            .execute_contract(
                sender.clone(),
                self.hub.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateRegistryFee { new_fee },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn update_deployer_addr(
        &mut self,
        sender: &Addr,
        new_addr: &str,
    ) -> Result<(), PRegistryContractError> {
        self.hub
            .app
            .execute_contract(
                sender.clone(),
                self.hub.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateDeployerAddr {
                    new_addr: new_addr.to_string(),
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn query_plugin(&self, id: u64) -> StdResult<Option<Plugin>> {
        let msg = PRegistryQueryMsg::GetPluginById { id };
        self.hub
            .app
            .wrap()
            .query_wasm_smart(self.hub.plugin_registry.clone(), &msg)
    }

    pub fn query_config(&self) -> StdResult<ConfigResponse> {
        let msg = PRegistryQueryMsg::GetConfig {};
        self.hub
            .app
            .wrap()
            .query_wasm_smart(self.hub.plugin_registry.clone(), &msg)
    }

    pub fn query_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<u32>,
    ) -> StdResult<PluginsResponse> {
        let msg = PRegistryQueryMsg::GetPlugins { limit, start_after };
        self.hub
            .app
            .wrap()
            .query_wasm_smart(self.hub.plugin_registry.clone(), &msg)
    }

    pub fn query_installed_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<String>,
    ) -> StdResult<PluginListResponse> {
        let msg = ProxyQueryMsg::Plugins { start_after, limit };
        self.hub
            .app
            .wrap()
            .query_wasm_smart(self.proxy.clone(), &msg)
    }
}
