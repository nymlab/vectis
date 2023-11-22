use crate::{helpers::default_entity, unit_tests::utils::*};

#[test]
fn must_create_with_correct_fees() {
    let suite = VectisTestSuite::new();

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: None,
        code_id: None,
    };

    let factory = VectisFactoryProxy::new(suite.factory, &suite.app);

    factory
        .factory_service_trait_proxy()
        .create_wallet(msg.clone())
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    let addr = factory
        .factory_service_trait_proxy()
        .wallet_by_vid("vectis-wallet".into())
        .unwrap();

    assert!(addr.is_some());

    // wrong fees does not create
    factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE + WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap_err();

    let total = factory
        .factory_management_trait_proxy()
        .total_created()
        .unwrap();

    assert_eq!(total, 1)
}

#[test]
fn must_be_deployer_to_create_wallet() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory, &suite.app);

    suite
        .app
        .app_mut()
        .send_tokens(
            Addr::unchecked(VALID_OSMO_ADDR),
            Addr::unchecked("other-addr"),
            &[coin(10000, DENOM)],
        )
        .unwrap();

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: None,
        code_id: None,
    };

    factory
        .factory_service_trait_proxy()
        .create_wallet(msg.clone())
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call("other-addr")
        .unwrap_err();
}

#[test]
fn must_not_create_wallet_with_same_vid() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory, &suite.app);

    let mut msg_with_vid = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: None,
        code_id: None,
    };

    factory
        .factory_service_trait_proxy()
        .create_wallet(msg_with_vid.clone())
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    // Same vid should fail
    factory
        .factory_service_trait_proxy()
        .create_wallet(msg_with_vid.clone())
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    msg_with_vid.vid = String::from("diff-vectis-wallet");
    factory
        .factory_service_trait_proxy()
        .create_wallet(msg_with_vid.clone())
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    assert!(factory
        .factory_service_trait_proxy()
        .wallet_by_vid("vectis-wallet".into())
        .unwrap()
        .is_some());

    assert!(factory
        .factory_service_trait_proxy()
        .wallet_by_vid("diff-vectis-wallet".into())
        .unwrap()
        .is_some());

    assert_eq!(
        factory
            .factory_management_trait_proxy()
            .total_created()
            .unwrap(),
        2
    );
}

#[test]
fn query_wallet_by_vid_chain_non_ibc_works() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory.clone(), &suite.app);

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: Some(vec![(NON_IBC_CHAIN_NAME.into(), NON_IBC_CHAIN_ADDR.into())]),
        code_id: None,
    };

    factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap();

    let wallet_addr = factory
        .factory_service_trait_proxy()
        .wallet_by_vid("vectis-wallet".into())
        .unwrap()
        .unwrap();

    let wallet = VectisProxyProxy::new(wallet_addr, &suite.app);

    let info: WalletInfo = wallet.wallet_trait_proxy().info().unwrap();
    assert_eq!(info.controller, default_entity());
    assert_eq!(info.deployer, suite.deployer);
    assert_eq!(info.vid, "vectis-wallet");
    assert_eq!(info.addresses.len(), 1);

    let remote_wallet_addr = factory
        .factory_service_trait_proxy()
        .wallet_by_vid_chain("vectis-wallet".into(), NON_IBC_CHAIN_NAME.into())
        .unwrap()
        .unwrap();

    assert_eq!(remote_wallet_addr, NON_IBC_CHAIN_ADDR);
}
