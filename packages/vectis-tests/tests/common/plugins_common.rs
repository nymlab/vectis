pub use crate::common::common::*;

/// PluginsSuite
///
/// PluginsSuite is a suite of contracts that are used to test the plugin registry
/// - registry

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PluginsSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    // The account that deploys everything
    pub deployer: Addr,
    // Dao address
    pub dao: Addr,
    // Controller
    pub controller: Addr,
    // plugin registry address
    pub plugin_registry: Addr,
    // factory address
    pub factory: Addr,
}

impl PluginsSuite {
    pub fn init() -> Result<PluginsSuite> {
        let genesis_funds = vec![coin(100000, "ucosm")];
        let deployer = Addr::unchecked("deployer");
        let dao: Addr = Addr::unchecked("dao");
        let controller = Addr::unchecked(CONTROLLER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(
            deployer.clone(),
            controller.clone(),
            &[coin(50000, "ucosm")],
        )?;

        let plugin_registry_id = app.store_code(contract_plugin_registry());
        let factory_id = app.store_code(contract_factory());
        let proxy_id = app.store_code(contract_proxy());
        let proxy_multisig_id = app.store_code(contract_multisig());

        let factory = app
            .instantiate_contract(
                factory_id,
                deployer.clone(),
                &InstantiateMsg {
                    proxy_code_id: proxy_id,
                    proxy_multisig_code_id: proxy_multisig_id,
                    addr_prefix: "wasm".to_string(),
                    wallet_fee: Coin {
                        denom: "ucosm".to_string(),
                        amount: Uint128::new(WALLET_FEE),
                    },
                    claim_fee: Coin {
                        denom: "ucosm".to_string(),
                        amount: Uint128::new(CLAIM_FEE),
                    },
                    govec_minter: Some(dao.to_string()),
                },
                &[],
                "wallet-factory",      // label: human readible name for contract
                Some(dao.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        let plugin_registry = app
            .instantiate_contract(
                plugin_registry_id,
                deployer.clone(),
                &PRegistryInstantiateMsg {
                    dao_addr: dao.to_string(),
                    registry_fee: coin(REGISTRY_FEE, "ucosm"),
                    reviewers: vec![deployer.to_string()],
                },
                &[],
                "plugin-registry",
                Some(dao.to_string()),
            )
            .unwrap();

        Ok(PluginsSuite {
            controller,
            app,
            dao,
            deployer,
            plugin_registry,
            factory,
        })
    }

    // Create wallet
    pub fn create_new_proxy_without_guardians(
        &mut self,
        controller: Addr,
        factory: Addr,
        proxy_initial_fund: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        self._create_new_proxy(
            controller,
            factory,
            proxy_initial_fund,
            guardians_multisig,
            vec![],
            native_tokens_amount,
        )
    }

    fn _create_new_proxy(
        &mut self,
        controller: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        guardians: Vec<String>,
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let r = "relayer".to_owned();

        let create_wallet_msg = CreateWalletMsg {
            controller_addr: controller.to_string(),
            guardians: Guardians {
                addresses: guardians,
                guardians_multisig,
            },
            relayers: vec![r],
            proxy_initial_funds,
            label: "initial label".to_string(),
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };

        let res = self
            .app
            .execute_contract(
                controller,
                factory,
                &execute,
                &[coin(native_tokens_amount, "ucosm")],
            )
            .map_err(|err| anyhow!(err))?;

        let wasm_events: Vec<Event> = res
            .events
            .iter()
            .cloned()
            .filter(|ev| ev.ty.as_str() == "wasm")
            .collect();

        // This event
        let ev = wasm_events
            .iter()
            .find(|event| event.attributes.iter().any(|at| at.key == "proxy_address"));

        let proxy = &ev.unwrap().attributes[2].value;

        Ok(Addr::unchecked(proxy))
    }

    pub fn query_balance(&self, addr: &Addr) -> Result<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), "ucosm")?)
    }

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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
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

    pub fn update_reviewers(
        &mut self,
        sender: &Addr,
        reviewers: Vec<String>,
    ) -> Result<(), PRegistryContractError> {
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
                &PRegistryExecMsg::UpdateReviewers { reviewers },
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
        self.app
            .execute_contract(
                sender.clone(),
                self.plugin_registry.clone(),
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

    pub fn query_plugins(
        &self,
        limit: Option<u32>,
        start_after: Option<u32>,
    ) -> StdResult<PluginsResponse> {
        let msg = PRegistryQueryMsg::GetPlugins { limit, start_after };
        self.app
            .wrap()
            .query_wasm_smart(self.plugin_registry.clone(), &msg)
    }
}
