use cosmwasm_std::{coin, to_binary};
use sylvia::multitest::App;

use vectis_proxy::wallet::contract::test_utils::WalletTrait;

use vectis_wallet::types::{
    authenticator::{Authenticator, AuthenticatorProvider},
    entity::Entity,
    factory::CreateWalletMsg,
    wallet::ProxyCreateMsg,
};

#[test]
fn proxy_instantiates_without_plugins() {
    let app = App::default();

    let factory = "factory";
    let proxy_code_id = vectis_proxy::contract::multitest_utils::CodeId::store_code(&app);
    let controller = Entity {
        auth: Authenticator {
            ty: vectis_wallet::types::authenticator::AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: to_binary("mock-data").unwrap(),
        nonce: 0,
    };
    let relayers = vec![];
    let proxy_initial_funds = vec![coin(100, "eur")];
    let vid = "test@vectis".into();
    let data_key = "some-key";
    let data_value = "some-value";
    let initial_data = vec![(to_binary(data_key).unwrap(), to_binary(data_value).unwrap())];

    proxy_code_id
        .instantiate(ProxyCreateMsg {
            create_wallet_msg: CreateWalletMsg {
                controller: controller.clone(),
                relayers: relayers.clone(),
                proxy_initial_funds,
                vid,
                initial_data,
                plugins: vec![],
            },
        })
        .with_label("Vectis Proxy")
        .call(factory)
        .unwrap();
}
