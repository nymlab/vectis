use cosmwasm_std::{Coin, Uint128};
use sylvia::multitest::App;

use vectis_factory::management::contract::test_utils::FactoryManagementTrait;
use vectis_wallet::types::factory::{
    ChainConnection, CodeIdType, FeeType, FeesResponse, WalletFactoryInstantiateMsg,
};

use crate::constants::VECTIS_VERSION;

#[test]
fn factory_instantiates_correctly_without_authenticators() {
    let app = App::default();
    let deployer = "deployer";
    let factory_code_id = vectis_factory::contract::multitest_utils::CodeId::store_code(&app);
    let proxy_code_id = vectis_proxy::contract::multitest_utils::CodeId::store_code(&app);
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
            supported_chains: Some(vec![(
                remote_chain_id.into(),
                remote_chain_connection.clone(),
            )]),
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
fn factory_can_update_configs() {
    let app = App::default();

    let deployer = "deployer";
    let factory_code_id = vectis_factory::contract::multitest_utils::CodeId::store_code(&app);
    let proxy_code_id = vectis_proxy::contract::multitest_utils::CodeId::store_code(&app);
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

    // Supported Chains
    // TODO:
}

#[test]
#[test]
fn update_proxy_code_id_as_expected() {
    let app = App::default();

    let deployer = "deployer";
    let factory_code_id = vectis_factory::contract::multitest_utils::CodeId::store_code(&app);
    let proxy_code_id = vectis_proxy::contract::multitest_utils::CodeId::store_code(&app);
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

    // Can Add new code id; Not default
    let new_code_id = 99;
    factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            new_code_id,
            Some("Updated-version".into()),
            false,
        )
        .call(deployer)
        .unwrap();

    let new_default_code_id = factory
        .factory_management_trait_proxy()
        .default_proxy_code_id()
        .unwrap();

    assert_eq!(new_default_code_id, old_default_code_id);

    let code_ids = factory
        .factory_management_trait_proxy()
        .supported_proxies(None, None)
        .unwrap();

    assert!(code_ids.contains(&(new_code_id, "Updated-version".into())));

    // Can Add new code id and set as default
    let new_default_code_id = 100;

    factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            new_default_code_id,
            Some("Updated-version".into()),
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

    assert!(code_ids.contains(&(new_default_code_id, "Updated-version".into())));

    // Can remove not default
    factory
        .factory_management_trait_proxy()
        .update_code_id(CodeIdType::Proxy, new_code_id, None, false)
        .call(deployer)
        .unwrap();

    let code_ids = factory
        .factory_management_trait_proxy()
        .supported_proxies(None, None)
        .unwrap();

    assert!(!code_ids.contains(&(new_code_id, "Updated-version".into())));

    // Cannot add if not deployer
    factory
        .factory_management_trait_proxy()
        .update_code_id(
            CodeIdType::Proxy,
            new_code_id,
            Some("Updated-version".into()),
            true,
        )
        .call("not-deployer")
        .unwrap_err();
}
