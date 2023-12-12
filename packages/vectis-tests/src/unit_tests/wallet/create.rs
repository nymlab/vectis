use crate::helpers::webauthn_entity;
use crate::unit_tests::utils::*;
use serial_test::serial;

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
