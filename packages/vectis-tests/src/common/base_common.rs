pub use crate::common::common::*;

/// This is initialised all the contracts
/// - factory: creates, upgrade proxy and store govec claim list
/// - plugin registry
/// - deployer: a multisig with some additional states
/// - plugin committee: a multisig
#[derive(Derivative)]
#[derivative(Debug)]
pub struct HubChainSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    // Controller
    pub controller: Addr,
    // factory address
    pub factory: Addr,
    // deployer address: cw3_flex_multisig
    pub deployer: Addr,
    // deployer signer: of the group
    pub deployer_signer: Addr,
    // deployer group cw4
    pub deployer_group: Addr,
    // PluginCommitte addr: test using single account
    pub plugin_committee: Addr,
    // PluginRegistry addr
    pub plugin_registry: Addr,
}

impl HubChainSuite {
    /// Instantiate factory contract with
    /// - no initial funds on the factory
    /// - default WALLET_FEE
    pub fn init() -> Result<HubChainSuite> {
        let genesis_funds = vec![coin(10_000_000_000_000, DENOM)];
        let deployer_signer = Addr::unchecked("signer");
        let plugin_committee = Addr::unchecked("plugin_committee");
        let controller = Addr::unchecked(CONTROLLER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer_signer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(
            deployer_signer.clone(),
            controller,
            &[coin(1_000_000_000, DENOM)],
        )?;

        app.send_tokens(
            deployer_signer.clone(),
            plugin_committee.clone(),
            &[coin(1_000_000_000, DENOM)],
        )?;

        let factory_id = app.store_code(contract_factory());
        let proxy_id = app.store_code(contract_proxy());
        let proxy_multisig_id = app.store_code(contract_fixed_multisig());
        let plugin_registry_id = app.store_code(contract_plugin_registry());
        let cw3_flex_id = app.store_code(contract_flex_multisig());
        let cw4_id = app.store_code(contract_cw4());

        let deployer_group = app
            .instantiate_contract(
                cw4_id,
                deployer_signer.clone(),
                &cw4InstMsg {
                    admin: None,
                    members: vec![Member {
                        addr: deployer_signer.to_string(),
                        weight: 1,
                    }],
                },
                &[],
                "Vectis Deployer Group",
                None,
            )
            .unwrap();

        let deployer = app
            .instantiate_contract(
                cw3_flex_id,
                deployer_signer.clone(),
                &cw3flexInstMsg {
                    group_addr: deployer_group.to_string(),
                    threshold: cw_utils::Threshold::AbsoluteCount { weight: 1 },
                    max_voting_period: Duration::Time(100),
                    executor: None,
                    proposal_deposit: None,
                },
                &[],
                "Vectis Deployer Multisig",
                None,
            )
            .unwrap();

        let factory_inst_msg = InstantiateMsg {
            proxy_code_id: proxy_id,
            proxy_multisig_code_id: proxy_multisig_id,
            addr_prefix: "wasm".to_string(),
            wallet_fee: Coin {
                denom: DENOM.to_string(),
                amount: Uint128::new(WALLET_FEE),
            },
        };

        let plugin_reg_inst_msg = PRegistryInstantiateMsg {
            registry_fee: coin(REGISTRY_FEE, DENOM),
            install_fee: coin(INSTALL_FEE, DENOM),
        };

        let factory = app
            .instantiate_contract(
                factory_id,
                deployer.clone(),
                &factory_inst_msg,
                &[],
                "Vectis Factory",
                Some(deployer.to_string()),
            )
            .unwrap();

        let plugin_registry = app
            .instantiate_contract(
                plugin_registry_id,
                deployer.clone(),
                &plugin_reg_inst_msg,
                &[],
                "Vectis Plugin Registry",
                Some(deployer.to_string()),
            )
            .unwrap();

        let actors = &[
            (VectisActors::Factory, factory.clone()),
            (VectisActors::PluginRegistry, plugin_registry.clone()),
            (VectisActors::PluginCommittee, plugin_committee.clone()),
        ];

        for actor in actors.iter() {
            app.execute_contract(
                deployer.clone(),
                deployer.clone(),
                &add_item_msg(actor.0.clone(), actor.1.clone()),
                &[],
            )
            .unwrap();
        }

        let hub_chain_suite = HubChainSuite {
            controller: Addr::unchecked(CONTROLLER_ADDR),
            app,
            deployer,
            factory,
            deployer_group,
            deployer_signer,
            plugin_registry,
            plugin_committee,
        };

        Ok(hub_chain_suite)
    }

    // Create wallet
    pub fn create_new_proxy_without_guardians(
        &mut self,
        controller: Addr,
        proxy_initial_funds: Vec<Coin>,
        wallet_fee: Coin,
    ) -> Result<Addr> {
        let native_tokens_amount = proxy_initial_funds
            .iter()
            .find(|c| c.denom == wallet_fee.denom)
            .map(|c| c.amount + wallet_fee.amount)
            .unwrap_or(wallet_fee.amount);

        self.create_new_proxy(
            controller,
            self.factory.clone(),
            proxy_initial_funds,
            None,
            vec![],
            native_tokens_amount.u128(),
        )
    }

    pub fn update_wallet_fee(&mut self, deployer: Addr, new_fee: Coin) -> Result<AppResponse> {
        self.app.execute_contract(
            deployer,
            self.factory.clone(),
            &FactoryExecuteMsg::UpdateConfigFee {
                ty: vectis_wallet::FeeType::Wallet,
                new_fee,
            },
            &[],
        )
    }

    pub fn create_new_proxy_with_default_guardians(
        &mut self,
        controller: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        wallet_fee: Coin,
    ) -> Result<Addr> {
        let g1 = Addr::unchecked(GUARD1);
        let g2 = Addr::unchecked(GUARD2);

        let native_tokens_amount = proxy_initial_funds
            .iter()
            .find(|c| c.denom == wallet_fee.denom)
            .map(|c| c.amount + wallet_fee.amount)
            .unwrap_or(wallet_fee.amount);

        self.create_new_proxy(
            controller,
            self.factory.clone(),
            proxy_initial_funds,
            guardians_multisig,
            vec![g1, g2],
            native_tokens_amount.u128(),
        )
    }

    pub fn create_new_proxy(
        &mut self,
        controller: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        guardians: Vec<Addr>,
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

        let mut fees = vec![];
        if native_tokens_amount != 0u128 {
            fees.push(coin(native_tokens_amount, DENOM))
        };
        let res = self
            .app
            .execute_contract(controller, factory, &execute, &fees)
            .map_err(|err| anyhow!(err))?;

        let wasm_events: Vec<Event> = res
            .events
            .iter()
            .cloned()
            .filter(|ev| ev.ty.as_str() == "wasm-vectis.proxy.v1.MsgInstantiate")
            .collect();

        // This event
        let attr = wasm_events[0]
            .attributes
            .iter()
            .find(|at| at.key == "_contract_addr");

        let proxy = &attr.unwrap().value;

        Ok(Addr::unchecked(proxy))
    }

    pub fn create_relay_transaction(
        &mut self,
        signer_sk: &[u8; 32],
        cosmos_msg: CosmosMsg,
        nonce: u64,
    ) -> RelayTransaction {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(signer_sk).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            controller_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        }
    }

    pub fn update_proxy_code_id(&mut self, new_code_id: u64, factory: Addr) -> Result<AppResponse> {
        self.app
            .execute_contract(
                self.deployer.clone(),
                factory,
                &FactoryExecuteMsg::UpdateCodeId {
                    ty: CodeIdType::Proxy,
                    new_code_id,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn update_proxy_multisig_code_id(
        &mut self,
        new_code_id: u64,
        factory: Addr,
    ) -> Result<AppResponse> {
        self.app
            .execute_contract(
                self.deployer.clone(),
                factory,
                &FactoryExecuteMsg::UpdateCodeId {
                    ty: CodeIdType::Multisig,
                    new_code_id,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn create_guardians_request_and_update_guardians(
        &mut self,
        sender: &Addr,
        wallet_address: &Addr,
        guardians: Guardians,
        new_multisig_code_id: Option<u64>,
    ) -> Result<AppResponse> {
        let request = GuardiansUpdateMsg {
            guardians,
            new_multisig_code_id,
        };

        let request_update_guardians_message: ProxyExecuteMsg =
            ProxyExecuteMsg::RequestUpdateGuardians {
                request: Some(request),
            };

        self.app
            .execute_contract(
                sender.clone(),
                wallet_address.clone(),
                &request_update_guardians_message,
                &[],
            )
            .unwrap();

        self.fast_forward_block_time(90000);

        let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {};

        let r = self
            .app
            .execute_contract(
                sender.clone(),
                wallet_address.clone(),
                &update_guardians_message,
                &[],
            )
            .unwrap();

        Ok(r)
    }

    pub fn proxy_execute(
        &mut self,
        wallet_addr: &Addr,
        msgs: Vec<CosmosMsg>,
        fees: Vec<Coin>,
    ) -> Result<AppResponse> {
        self.app.execute_contract(
            self.controller.clone(),
            wallet_addr.clone(),
            &ProxyExecuteMsg::Execute { msgs },
            &fees,
        )
    }

    pub fn fast_forward_block_time(&mut self, forward_time_sec: u64) {
        let block = self.app.block_info();

        let mock_block = BlockInfo {
            height: block.height + 10,
            chain_id: block.chain_id,
            time: block.time.plus_seconds(forward_time_sec),
        };

        self.app.set_block(mock_block);
    }

    pub fn query_plugins(&self, contract_addr: &Addr) -> Result<PluginListResponse, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ProxyQueryMsg::Plugins {}).unwrap(),
        }))
    }

    pub fn query_wallet_info<R>(&self, contract_addr: &Addr) -> Result<R, StdError>
    where
        R: DeserializeOwned,
    {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ProxyQueryMsg::Info {}).unwrap(),
        }))
    }

    pub fn query_controller_wallet(&self, controller: Addr) -> Result<Vec<Addr>, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.factory.to_string(),
            msg: to_binary(&FactoryQueryMsg::ControllerWallets { controller }).unwrap(),
        }))
    }

    pub fn query_wallets_with_guardian(&self, guardian: Addr) -> Result<Vec<Addr>, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: self.factory.to_string(),
            msg: to_binary(&FactoryQueryMsg::WalletsWithGuardian { guardian }).unwrap(),
        }))
    }

    pub fn query_guardians_request(
        &self,
        contract_addr: &Addr,
    ) -> Result<Option<GuardiansUpdateRequest>, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ProxyQueryMsg::GuardiansUpdateRequest {}).unwrap(),
        }))
    }

    pub fn query_multisig_voters(
        &self,
        contract_addr: &Addr,
    ) -> Result<VoterListResponse, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&MultiSigQueryMsg::ListVoters {
                start_after: None,
                limit: None,
            })
            .unwrap(),
        }))
    }

    pub fn query_balance(&self, addr: &Addr) -> Result<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), DENOM)?)
    }
}

pub fn add_item_msg(key: VectisActors, value: Addr) -> cw3flexExecMsg {
    cw3flexExecMsg::UpdateItem {
        key: format!("{key}"),
        value: value.to_string(),
    }
}

pub fn init_funds() -> Vec<Coin> {
    vec![coin(10_000_000_000_000, DENOM)]
}
