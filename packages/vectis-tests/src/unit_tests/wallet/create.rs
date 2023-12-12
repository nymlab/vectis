use crate::helpers::webauthn_entity;
use crate::unit_tests::utils::*;
use serial_test::serial;
use vectis_wallet::types::error::FactoryError;

#[test]
#[serial]
fn test_create_wallet_with_passkey() {
    let suite = VectisTestSuite::new();

    let pubkey = must_create_credential("vectis-wallet");
    let entity = webauthn_entity(&pubkey);
    let wallet_addr = suite.create_default_wallet(entity, "vectis-wallet".into(), vec![]);

    let wallet = VectisProxyProxy::new(wallet_addr, &suite.app);
    let info: WalletInfo = wallet.wallet_trait_proxy().info().unwrap();

    assert_eq!(info.controller.data.0, pubkey);
}

// This error message cannot be downcasted from multitest
// https://github.com/CosmWasm/sylvia/blob/b3cc2c589166c9b2d26657b4663093918fdb2ad3/sylvia/src/multitest.rs#L149
// Therefore we are currently ensuring at at last the `1: Duplication relayers` is in there to
// ensure it is the correct error
#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: Error executing WasmMsg:\n  sender: osmo1hm4y6fzgxgu688jgf7ek66px6xkrtmn3gyk8fax3eawhp68c2d5qwud8uh\n  Execute { contract_addr: \"osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\", msg: {\"create_wallet\":{\"create_wallet_msg\":{\"controller\":{\"auth\":{\"ty\":\"webauthn\",\"provider\":\"vectis\"},\"data\":\"ImRhdGEi\",\"nonce\":0},\"relayers\":[\"relayer1\",\"relayer2\",\"relayer1\"],\"proxy_initial_funds\":[],\"vid\":\"vectis-wallet\",\"initial_data\":[],\"plugins\":[],\"chains\":null,\"code_id\":null}}}, funds: [Coin { 10 \"uosmo\" }] }\n\nCaused by:\n    0: Error executing WasmMsg:\n         sender: osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\n         Instantiate2 { admin: Some(\"osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\"), code_id: 3, label: \"Vectis-Proxy\", msg: {\"msg\":{\"create_wallet_msg\":{\"controller\":{\"auth\":{\"ty\":\"webauthn\",\"provider\":\"vectis\"},\"data\":\"ImRhdGEi\",\"nonce\":0},\"relayers\":[\"relayer1\",\"relayer2\",\"relayer1\"],\"proxy_initial_funds\":[],\"vid\":\"vectis-wallet\",\"initial_data\":[],\"plugins\":[],\"chains\":null,\"code_id\":null}}}, funds: [], salt: Binary(31) }\n    1: Duplication relayers"
)]
fn cannot_create_with_duplicated_relayers() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory.clone(), &suite.app);

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![
            "relayer1".to_string(),
            "relayer2".to_string(),
            "relayer1".to_string(),
        ],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: None,
        code_id: None,
    };

    let _ = factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str());
}
#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: Error executing WasmMsg:\n  sender: osmo1hm4y6fzgxgu688jgf7ek66px6xkrtmn3gyk8fax3eawhp68c2d5qwud8uh\n  Execute { contract_addr: \"osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\", msg: {\"create_wallet\":{\"create_wallet_msg\":{\"controller\":{\"auth\":{\"ty\":\"webauthn\",\"provider\":\"vectis\"},\"data\":\"ImRhdGEi\",\"nonce\":0},\"relayers\":[],\"proxy_initial_funds\":[],\"vid\":\"vectis-wallet\",\"initial_data\":[],\"plugins\":[],\"chains\":[[\"non-ibc-chain-1\",\"ChainInfo-1\"],[\"non-ibc-chain-1\",\"ChainInfo-1\"]],\"code_id\":null}}}, funds: [Coin { 10 \"uosmo\" }] }\n\nCaused by:\n    0: Error executing WasmMsg:\n         sender: osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\n         Instantiate2 { admin: Some(\"osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv\"), code_id: 3, label: \"Vectis-Proxy\", msg: {\"msg\":{\"create_wallet_msg\":{\"controller\":{\"auth\":{\"ty\":\"webauthn\",\"provider\":\"vectis\"},\"data\":\"ImRhdGEi\",\"nonce\":0},\"relayers\":[],\"proxy_initial_funds\":[],\"vid\":\"vectis-wallet\",\"initial_data\":[],\"plugins\":[],\"chains\":[[\"non-ibc-chain-1\",\"ChainInfo-1\"],[\"non-ibc-chain-1\",\"ChainInfo-1\"]],\"code_id\":null}}}, funds: [], salt: Binary(31) }\n    1: Duplication required_chains"
)]
fn cannot_create_with_duplicated_required_chains() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory.clone(), &suite.app);

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: Some(vec![
            (NON_IBC_CHAIN_NAME.into(), String::from("ChainInfo-1")),
            (NON_IBC_CHAIN_NAME.into(), String::from("ChainInfo-1")),
        ]),
        code_id: None,
    };

    let _ = factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap_err();
}

#[test]
fn cannot_create_with_duplicated_proxy_init_fund() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory.clone(), &suite.app);

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![coin(11, DENOM), coin(22, DENOM)],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: None,
        code_id: None,
    };

    let err = factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE + 11 + 22, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap_err();

    assert_eq!(err, FactoryError::Duplication("proxy_init_funds".into()));
}

#[test]
fn cannot_create_with_unsupported_code_id() {
    let suite = VectisTestSuite::new();
    let factory = VectisFactoryProxy::new(suite.factory.clone(), &suite.app);

    let unsupported_code_id = ProxyCodeId::store_code(&suite.app);

    let msg = CreateWalletMsg {
        controller: default_entity(),
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid: String::from("vectis-wallet"),
        initial_data: vec![],
        plugins: vec![],
        chains: Some(vec![]),
        // Unsupported_code_id
        code_id: Some(unsupported_code_id.code_id()),
    };

    let err = factory
        .factory_service_trait_proxy()
        .create_wallet(msg)
        .with_funds(&[coin(WALLET_FEE, DENOM)])
        .call(suite.deployer.as_str())
        .unwrap_err();

    assert_eq!(err, FactoryError::NotSupportedProxyCodeId)
}
