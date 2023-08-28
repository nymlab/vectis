use cosmwasm_std::{Coin, Uint128};
use sylvia::multitest::App;

use vectis_factory::management::contract::test_utils::FactoryManagementTrait;
use vectis_wallet::types::factory::{
    CodeIdType, FeeType, FeesResponse, WalletFactoryInstantiateMsg,
};

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

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            proxy_code_id: proxy_code_id.code_id(),
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    let code_id = factory
        .factory_management_trait_proxy()
        .code_id(CodeIdType::Proxy)
        .unwrap();

    let fees = factory.factory_management_trait_proxy().fees().unwrap();

    let actual_deployer = factory.factory_management_trait_proxy().deployer().unwrap();
    let total_create = factory
        .factory_management_trait_proxy()
        .total_created()
        .unwrap();

    assert_eq!(code_id, proxy_code_id.code_id());
    assert_eq!(fees, FeesResponse { wallet_fee });
    assert_eq!(total_create, 0u64);
    assert_eq!(actual_deployer, deployer);
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

    let factory = factory_code_id
        .instantiate(WalletFactoryInstantiateMsg {
            proxy_code_id: proxy_code_id.code_id(),
            wallet_fee: wallet_fee.clone(),
            authenticators: None,
        })
        .with_label("Vectis Factory")
        .call(deployer)
        .unwrap();

    // Code ID
    let new_code_id = 99;
    factory
        .factory_management_trait_proxy()
        .update_code_id(CodeIdType::Proxy, new_code_id)
        .call(deployer)
        .unwrap();

    factory
        .factory_management_trait_proxy()
        .update_code_id(CodeIdType::Proxy, new_code_id)
        .call("invalid_deployer")
        .unwrap_err();

    let actual_id = factory
        .factory_management_trait_proxy()
        .code_id(CodeIdType::Proxy)
        .unwrap();

    assert_ne!(proxy_code_id.code_id(), actual_id);
    assert_eq!(new_code_id, actual_id);

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
}
