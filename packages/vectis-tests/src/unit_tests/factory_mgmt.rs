use vectis_wallet::types::error::FactoryError;

use crate::unit_tests::utils::*;

#[test]
fn factory_instantiates_correctly_without_authenticators() {
    let app = App::default();
    let deployer = VALID_OSMO_ADDR;
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };
    let remote_chain_id = "remote_chain_id";
    let remote_chain_connection = ChainConnection::IBC("connection-id-1".into());

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: Some(vec![(
                remote_chain_id.into(),
                remote_chain_connection.clone(),
            )]),
            wallet_creator: deployer.into(),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    let code_id = factory
        .factory_management_trait_proxy()
        .default_proxy_code_id()
        .unwrap();

    let fees = factory.factory_management_trait_proxy().fees().unwrap();

    let actual_deployer = factory.factory_management_trait_proxy().deployer().unwrap();
    let total_create = factory
        .factory_management_trait_proxy()
        .total_created()
        .unwrap();

    let total_supported_chains = factory
        .factory_management_trait_proxy()
        .supported_chains(None, None)
        .unwrap();

    assert_eq!(code_id, proxy_code_id.code_id());
    assert_eq!(fees, FeesResponse { wallet_fee });
    assert_eq!(total_create, 0u64);
    assert_eq!(actual_deployer, deployer);
    assert_eq!(
        total_supported_chains,
        vec![(remote_chain_id.into(), remote_chain_connection)]
    );
}

#[test]
fn factory_instantiates_correctly_with_authenticators_and_updates() {
    let mtapp = AppBuilder::default()
        .with_api(MockApiBech32::new("osmo"))
        .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
        .build(|_, _, _| {});

    let app = App::new(mtapp);
    let deployer = VALID_OSMO_ADDR;
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let auth_code_id = AuthCodeId::store_code(&app);

    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };
    let remote_chain_id = "remote_chain_id";
    let remote_chain_connection = ChainConnection::IBC("connection-id-1".into());
    let authenticator = AuthenticatorInstInfo {
        ty: AuthenticatorType::Webauthn,
        code_id: auth_code_id.code_id(),
        inst_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
    };

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            // authenticators: None,
            authenticators: Some(vec![authenticator]),
            supported_chains: Some(vec![(
                remote_chain_id.into(),
                remote_chain_connection.clone(),
            )]),
            wallet_creator: deployer.into(),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    let auth_provider_addr_1 = factory
        .factory_management_trait_proxy()
        .auth_provider_addr(AuthenticatorType::Webauthn)
        .unwrap()
        .unwrap();

    factory
        .factory_management_trait_proxy()
        .update_auth_provider(AuthenticatorType::Webauthn, None, None)
        .call(deployer)
        .unwrap();

    let auth_provider_addr_2 = factory
        .factory_management_trait_proxy()
        .auth_provider_addr(AuthenticatorType::Webauthn)
        .unwrap();

    assert!(auth_provider_addr_2.is_none());

    app.set_block(BlockInfo {
        height: 2,
        time: Timestamp::from_seconds(201),
        chain_id: "Vectis".into(),
    });

    factory
        .factory_management_trait_proxy()
        .update_auth_provider(
            AuthenticatorType::Webauthn,
            Some(auth_code_id.code_id()),
            Some(to_binary(&EmptyInstantiateMsg {}).unwrap()),
        )
        .call(deployer)
        .unwrap();

    let auth_provider_addr_3 = factory
        .factory_management_trait_proxy()
        .auth_provider_addr(AuthenticatorType::Webauthn)
        .unwrap()
        .unwrap();

    assert_ne!(auth_provider_addr_1, auth_provider_addr_3);
}

#[test]
fn factory_can_update_configs() {
    let app = App::default();

    let deployer = "deployer";
    let factory_code_id = vectis_factory::contract::sv::multitest_utils::CodeId::store_code(&app);
    let proxy_code_id = vectis_proxy::contract::sv::multitest_utils::CodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: "uvectis".into(),
        amount: Uint128::one(),
    };
    let remote_chain_id = "remote_chain_id";
    let remote_chain_connection = ChainConnection::IBC("connection-id-1".into());

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: Some(vec![(remote_chain_id.into(), remote_chain_connection)]),
            wallet_creator: deployer.into(),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    // Fees
    let new_wallet_fee = Coin {
        denom: "uvectis".into(),
        amount: Uint128::from(123u128),
    };
    factory
        .factory_management_trait_proxy()
        .update_config_fee(FeeType::Wallet, new_wallet_fee.clone())
        .call(deployer)
        .unwrap();

    factory
        .factory_management_trait_proxy()
        .update_config_fee(FeeType::Wallet, new_wallet_fee.clone())
        .call("invalid_deployer")
        .unwrap_err();

    let fees = factory.factory_management_trait_proxy().fees().unwrap();

    assert_ne!(fees.wallet_fee, wallet_fee);
    assert_eq!(fees.wallet_fee, new_wallet_fee);

    // Supported Chains
    let new_chain_id = "new_chain_id";
    let new_connection = ChainConnection::Other("Some-other-connection".into());

    // add new supported chain
    factory
        .factory_management_trait_proxy()
        .update_supported_interchain(new_chain_id.into(), Some(new_connection))
        .call(deployer)
        .unwrap();

    let supported_chains = factory
        .factory_management_trait_proxy()
        .supported_chains(None, None)
        .unwrap();
    assert_eq!(supported_chains.len(), 2);

    // remove existing supported chain
    factory
        .factory_management_trait_proxy()
        .update_supported_interchain(new_chain_id.into(), None)
        .call(deployer)
        .unwrap();

    // Deployer
    let new_deployer = String::from("test-new-deployer");
    factory
        .factory_management_trait_proxy()
        .update_deployer(new_deployer.clone())
        .call(deployer)
        .unwrap();

    // old deployer cannot update deployer
    factory
        .factory_management_trait_proxy()
        .update_deployer(new_deployer.clone())
        .call(deployer)
        .unwrap_err();

    let actual_deployer = factory.factory_management_trait_proxy().deployer().unwrap();
    assert_ne!(actual_deployer.as_str(), deployer);
    assert_eq!(actual_deployer.as_str(), new_deployer);
}

#[test]
fn update_proxy_code_id_as_expected() {
    let app = App::default();

    let deployer = "deployer";
    let factory_code_id = vectis_factory::contract::sv::multitest_utils::CodeId::store_code(&app);
    let proxy_code_id = vectis_proxy::contract::sv::multitest_utils::CodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };
    let remote_chain_id = "remote_chain_id";
    let remote_chain_connection = ChainConnection::IBC("connection-id-1".into());

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: Some(vec![(remote_chain_id.into(), remote_chain_connection)]),
            wallet_creator: deployer.into(),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    let old_default_code_id = factory
        .factory_management_trait_proxy()
        .default_proxy_code_id()
        .unwrap();

    // Cannot remove default
    factory
        .factory_management_trait_proxy()
        .update_code_id(CodeIdType::Proxy, proxy_code_id.code_id(), None, false)
        .call(deployer)
        .unwrap_err();

    // Cannot add new code id; Not onchain
    let not_onchain_code_id = 99;
    let err = factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            not_onchain_code_id,
            Some("failed_version".into()),
            false,
        )
        .call(deployer)
        .unwrap_err();

    assert_eq!(err, FactoryError::ProxyCodeIdNotOnChain);

    // add new code_id onchain
    let new_default_code_id = vectis_proxy::contract::sv::multitest_utils::CodeId::store_code(&app);
    let new_default_code_id = new_default_code_id.code_id();

    // Can Add new code id and set as default
    factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            new_default_code_id,
            Some("Updated-default-version".into()),
            true,
        )
        .call(deployer)
        .unwrap();

    let actual_default_code_id = factory
        .factory_management_trait_proxy()
        .default_proxy_code_id()
        .unwrap();

    assert_eq!(actual_default_code_id, new_default_code_id);
    assert_ne!(actual_default_code_id, old_default_code_id);

    let code_ids = factory
        .factory_management_trait_proxy()
        .supported_proxies(None, None)
        .unwrap();

    assert!(code_ids.contains(&(new_default_code_id, "Updated-default-version".into())));

    // Can remove not default
    factory
        .factory_management_trait_proxy()
        .update_code_id(CodeIdType::Proxy, proxy_code_id.code_id(), None, false)
        .call(deployer)
        .unwrap();

    let code_ids = factory
        .factory_management_trait_proxy()
        .supported_proxies(None, None)
        .unwrap();

    assert!(!code_ids.contains(&(proxy_code_id.code_id(), VECTIS_VERSION.into())));

    // Cannot add if not deployer
    let err = factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            proxy_code_id.code_id(),
            Some(VECTIS_VERSION.into()),
            true,
        )
        .call("not-deployer")
        .unwrap_err();

    assert_eq!(err, FactoryError::Unauthorized)
}

#[test]
fn factory_instantiates_error_on_duplicated_supported_chains() {
    let app = App::default();
    let deployer = VALID_OSMO_ADDR;
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };
    let remote_chain_id = "remote_chain_id";
    let remote_chain_connection = ChainConnection::IBC("connection-id-1".into());

    let err = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: Some(vec![
                (remote_chain_id.into(), remote_chain_connection.clone()),
                (remote_chain_id.into(), remote_chain_connection.clone()),
            ]),
            wallet_creator: deployer.into(),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap_err();

    assert_eq!(err, FactoryError::Duplication("supported_chains".into()))
}

#[test]
fn factory_instantiates_error_on_zero_supported_proxies() {
    let app = App::default();
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };

    let err = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: None,
            wallet_creator: VALID_OSMO_ADDR.into(),
        })
        .with_label("Vectis Factory")
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    assert_eq!(err, FactoryError::InstantiationWithoutProxy)
}

#[test]
fn factory_instantiates_error_on_no_default_id_in_supported_proxies() {
    let app = App::default();
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };

    let err = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            // Does not include the default proxy id
            supported_proxies: vec![(1837, VECTIS_VERSION.into())],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: None,
            wallet_creator: VALID_OSMO_ADDR.into(),
        })
        .with_label("Vectis Factory")
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    assert_eq!(err, FactoryError::IncorrectDefaultProxy)
}

#[test]
fn factory_instantiates_error_on_duplicated_proxies() {
    let app = App::default();
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    let wallet_fee = Coin {
        denom: DENOM.into(),
        amount: WALLET_FEE.into(),
    };

    let err = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![
                (proxy_code_id.code_id(), VECTIS_VERSION.into()),
                (proxy_code_id.code_id(), VECTIS_VERSION.into()),
            ],
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
            supported_chains: None,
            wallet_creator: VALID_OSMO_ADDR.into(),
        })
        .with_label("Vectis Factory")
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    assert_eq!(err, FactoryError::Duplication("supported_proxies".into()))
}
