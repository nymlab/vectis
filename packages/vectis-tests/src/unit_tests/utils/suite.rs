use super::*;
use cw_multi_test::Executor;

pub struct VectisTestSuite {
    pub app: App<cw_multi_test::App<cw_multi_test::BankKeeper, MockApiBech32>>,
    // deployer contract (where Items are stored)
    pub deployer: Addr,
    // deployer signer: of the group
    pub deployer_signer: Addr,
    // deployer group cw4
    pub deployer_group: Addr,
    // Controller
    pub controller: Entity,
    // factory address
    pub factory: Addr,
    // PluginRegistry addr
    pub plugin_registry: Addr,
}

impl VectisTestSuite {
    pub fn new() -> Self {
        let mtapp = new_mtapp();
        let app = App::new(mtapp);

        let deployer_signer = Addr::unchecked(VALID_OSMO_ADDR);
        let factory_code_id = FactoryCodeId::store_code(&app);
        let plugin_reg_code_id = RegistryCodeId::store_code(&app);
        let proxy_code_id = ProxyCodeId::store_code(&app);
        let auth_code_id = AuthCodeId::store_code(&app);
        let cw3_flex_id = app.app_mut().store_code(contract_flex_multisig());
        let cw4_id = app.app_mut().store_code(contract_cw4());

        let deployer_group = app
            .app_mut()
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
            .app_mut()
            .instantiate_contract(
                cw3_flex_id,
                deployer_signer.clone(),
                &cw3flexInstMsg {
                    group_addr: deployer_group.to_string(),
                    threshold: cw_utils::Threshold::AbsoluteCount { weight: 1 },
                    max_voting_period: cw_utils::Duration::Time(100),
                    executor: None,
                    proposal_deposit: None,
                },
                &[],
                "Vectis Deployer Multisig",
                None,
            )
            .unwrap();

        let factory_inst_msg = vectis_factory::contract::sv::InstantiateMsg {
            msg: WalletFactoryInstantiateMsg {
                default_proxy_code_id: proxy_code_id.code_id(),
                supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
                wallet_fee: coin(WALLET_FEE, DENOM),
                authenticators: Some(vec![AuthenticatorInstInfo {
                    ty: AuthenticatorType::Webauthn,
                    code_id: auth_code_id.code_id(),
                    inst_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
                }]),
                supported_chains: Some(vec![
                    (
                        NON_IBC_CHAIN_NAME.into(),
                        ChainConnection::Other(NON_IBC_CHAIN_CONN.into()).clone(),
                    ),
                    // Not able to do ibc query in multitest
                    //   (
                    //       REMOTE_IBC_CHAIN_ID.into(),
                    //       ChainConnection::IBC(REMOTE_IBC_CHAIN_CONNECTION.into()),
                    //   ),
                ]),
            },
        };

        let subscription_tiers = vec![(SubscriptionTier::Free, tier_0())];
        let plugin_reg_inst_msg = vectis_plugin_registry::contract::sv::InstantiateMsg {
            subscription_tiers,
            registry_fee: coin(REGISTRY_FEE, DENOM),
        };

        let factory = app
            .app_mut()
            .instantiate_contract(
                factory_code_id.code_id(),
                deployer.clone(),
                &factory_inst_msg,
                &[],
                "Vectis Factory",
                Some(deployer.to_string()),
            )
            .unwrap();

        let plugin_registry = app
            .app_mut()
            .instantiate_contract(
                plugin_reg_code_id.code_id(),
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
        ];

        for actor in actors.iter() {
            app.app_mut()
                .execute_contract(
                    deployer.clone(),
                    deployer.clone(),
                    &add_item_msg(actor.0.clone(), actor.1.clone()),
                    &[],
                )
                .unwrap();
        }

        app.app_mut()
            .send_tokens(
                Addr::unchecked(VALID_OSMO_ADDR),
                deployer.clone(),
                &[coin(100000, DENOM)],
            )
            .unwrap();

        Self {
            app,
            factory,
            controller: default_entity(),
            plugin_registry,
            deployer,
            deployer_group,
            deployer_signer,
        }
    }

    pub fn create_default_wallet(&self, entity: Entity, vid: String) -> Addr {
        let msg = CreateWalletMsg {
            controller: entity,
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid,
            initial_data: vec![],
            plugins: vec![],
            chains: None,
            code_id: None,
        };

        let factory = VectisFactoryProxy::new(self.factory.clone(), &self.app);

        factory
            .factory_service_trait_proxy()
            .create_wallet(msg.clone())
            .with_funds(&[coin(WALLET_FEE, DENOM)])
            .call(self.deployer.as_str())
            .unwrap();

        factory
            .factory_service_trait_proxy()
            .wallet_by_vid("vectis-wallet".into())
            .unwrap()
            .unwrap()
    }
}

pub fn new_mtapp() -> cw_multi_test::App<BankKeeper, MockApiBech32> {
    return AppBuilder::default()
        .with_api(MockApiBech32::new("osmo"))
        .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(VALID_OSMO_ADDR),
                    vec![coin(1000000000, DENOM)],
                )
                .unwrap();
        });
}
