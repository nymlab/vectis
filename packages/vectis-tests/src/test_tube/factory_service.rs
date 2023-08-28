use super::{contracts::Contract, test_env::HubChainSuite};
use cosmwasm_std::Addr;
use osmosis_test_tube::{Account, OsmosisTestApp};

use vectis_wallet::{
    interface::factory_management_trait::QueryMsg as FactoryQueryMsg,
    types::authenticator::AuthenticatorType,
};

#[test]
fn test_setup() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let factory = Contract::from_addr(&app, suite.factory);
    let auth_provide: Addr = factory
        .query(&FactoryQueryMsg::AuthProviderAddr {
            ty: AuthenticatorType::Webauthn,
        })
        .unwrap();
    assert_eq!(suite.webauthn, auth_provide.to_string())
}
