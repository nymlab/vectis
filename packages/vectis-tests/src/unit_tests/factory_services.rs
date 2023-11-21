use cw_multi_test::Executor;
use vectis_factory::service::contract::sv::test_utils::FactoryServiceTrait;
use vectis_wallet::types::{factory::CreateWalletMsg, wallet::Controller};

use crate::unit_tests::utils::*;

const remote_ibc_chain_id: &str = "remote_ibc_chain_id";
const remote_ibc_chain_connection: &str = "ibc-connection-1";
const remote_chain_id: &str = "remote_chain_id";
const remote_chain_connection: &str = "other-connection-id-1";

fn deploy_factory<'a>(
    app: &'a App<cw_multi_test::App<cw_multi_test::BankKeeper, MockApiBech32>>,
) -> vectis_factory::contract::sv::multitest_utils::VectisFactoryProxy<
    'a,
    cw_multi_test::App<cw_multi_test::BankKeeper, MockApiBech32>,
> {
    let deployer = VALID_OSMO_ADDR;
    let factory_code_id = FactoryCodeId::store_code(&app);
    let proxy_code_id = ProxyCodeId::store_code(&app);
    //let auth_code_id = AuthCodeId::store_code(&app);

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id.code_id(),
            supported_proxies: vec![(proxy_code_id.code_id(), VECTIS_VERSION.into())],
            wallet_fee: coin(WALLET_FEE, DENOM),
            authenticators: None,
            supported_chains: Some(vec![
                (
                    remote_chain_id.into(),
                    ChainConnection::Other(remote_ibc_chain_connection.into()).clone(),
                ),
                (
                    remote_ibc_chain_id.into(),
                    ChainConnection::IBC(remote_ibc_chain_connection.into()),
                ),
            ]),
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();
    return factory;
}

fn new_mtapp() -> cw_multi_test::App<BankKeeper, MockApiBech32> {
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

#[test]
fn must_create_with_correct_fees() {
    let mtapp = new_mtapp();
    let app = App::new(mtapp);

    let factory = deploy_factory(&app);

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
        .call(VALID_OSMO_ADDR)
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
        .call(VALID_OSMO_ADDR)
        .unwrap_err();

    let total = factory
        .factory_management_trait_proxy()
        .total_created()
        .unwrap();

    assert_eq!(total, 1)
}

#[test]
fn must_be_deployer_to_create_wallet() {
    let mtapp = new_mtapp();
    let app = App::new(mtapp);

    app.app_mut()
        .send_tokens(
            Addr::unchecked(VALID_OSMO_ADDR),
            Addr::unchecked("other-addr"),
            &[coin(10000, DENOM)],
        )
        .unwrap();

    let factory = deploy_factory(&app);

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
