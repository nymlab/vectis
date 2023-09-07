pub use crate::common::base_common::*;
pub use crate::common::common::*;

/// Useful methods for registry
impl HubChainSuite {
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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
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
            self.deployer.to_string(),
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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
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
        creator: Option<String>,
        ipfs_hash: Option<String>,
        checksum: Option<String>,
        version: String,
    ) -> Result<(), PRegistryContractError> {
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
                &PRegistryExecMsg::UpdatePlugin {
                    id,
                    code_id,
                    creator,
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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateRegistryFee { new_fee },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn update_install_fee(
        &mut self,
        sender: &Addr,
        new_fee: Coin,
    ) -> Result<(), PRegistryContractError> {
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateInstallFee { new_fee },
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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
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
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }

    pub fn query_plugin_info(&self, addr: &Addr) -> StdResult<PluginWithVersionResponse> {
        let msg = PRegistryQueryMsg::QueryPluginByAddress {
            contract_addr: addr.to_string(),
        };
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }

    pub fn query_config(&self) -> StdResult<ConfigResponse> {
        let msg = PRegistryQueryMsg::GetConfig {};
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }

    pub fn query_fees(&self) -> StdResult<PRegistryFees> {
        let msg = PRegistryQueryMsg::GetFees {};
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }

    pub fn query_registered_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<u32>,
    ) -> StdResult<PluginsResponse> {
        let msg = PRegistryQueryMsg::GetPlugins { limit, start_after };
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }

    pub fn query_installed_plugins(&self, proxy: &Addr) -> StdResult<PluginListResponse> {
        let msg = ProxyQueryMsg::Plugins {};
        self.app.wrap().query_wasm_smart(proxy.clone(), &msg)
    }
}
