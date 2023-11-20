use cosmwasm_std::Addr;
use osmosis_test_tube::OsmosisTestApp;

use vectis_wallet::{
    interface::factory_management_trait::sv::QueryMsg as FactoryMgmtQueryMsg,
    types::authenticator::AuthenticatorType,
};

use crate::test_tube::{test_env::HubChainSuite, util::contract::Contract};

#[test]
fn create_factory_with_correct_authenticator() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let factory = Contract::from_addr(&app, suite.factory);
    let auth_provide: Addr = factory
        .query(&FactoryMgmtQueryMsg::AuthProviderAddr {
            ty: AuthenticatorType::Webauthn,
        })
        .unwrap();

    assert_eq!(suite.webauthn, auth_provide.to_string())
}
