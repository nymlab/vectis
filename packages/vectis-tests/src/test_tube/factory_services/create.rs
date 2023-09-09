use cosmwasm_std::{coin, to_binary, Addr, Binary};
use osmosis_test_tube::OsmosisTestApp;

use vectis_wallet::{
    interface::{
        factory_management_trait::QueryMsg as FactoryMgmtQueryMsg,
        factory_service_trait::{
            ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
        },
        wallet_trait::QueryMsg as WalletQueryMsg,
    },
    types::{
        authenticator::AuthenticatorType,
        factory::CreateWalletMsg,
        wallet::{WalletAddrs, WalletInfo},
    },
};

use crate::test_tube::{
    test_env::HubChainSuite,
    util::{constants::*, contract::Contract, wallet::default_entity},
};

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

#[test]
fn create_wallet_successfully_without_relayer() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let remote_chain_addr_in_base_64 = to_binary(NON_IBC_CHAIN_ADDR).unwrap();
    let initial_data = (
        to_binary(NON_IBC_CHAIN_NAME).unwrap(),
        remote_chain_addr_in_base_64.clone(),
    );

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            // NOTE: we cannot test feegrant on osmosis-test-tube as it is not registered
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid: vid.clone(),
            initial_data: vec![initial_data.clone()],
            plugins: vec![],
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    let res = factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, "uosmo")],
            &suite.accounts[IDEPLOYER],
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid.clone() })
        .unwrap();

    let total_created: u64 = factory
        .query(&FactoryMgmtQueryMsg::TotalCreated {})
        .unwrap();

    assert_eq!(total_created, 1);
    assert!(res
        .events
        .into_iter()
        .find(|e| e.ty == "wasm-vectis.factory.v1")
        .is_some());

    let wallet = Contract::from_addr(&app, wallet_addr.unwrap().to_string());

    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();
    let data: Option<Binary> = wallet
        .query(&WalletQueryMsg::Data {
            key: initial_data.0,
        })
        .unwrap();

    assert_eq!(info.deployer, suite.deployer);
    assert_eq!(info.controller, entity);
    assert_eq!(data.unwrap(), initial_data.1);
    assert!(info.relayers.is_empty());
    assert_eq!(info.vid, vid);
    assert_eq!(
        info.addresses,
        vec![
            WalletAddrs {
                chain_id: IBC_CHAIN_NAME.into(),
                addr: None
            },
            WalletAddrs {
                chain_id: NON_IBC_CHAIN_NAME.into(),
                addr: Some(remote_chain_addr_in_base_64.to_string())
            },
        ]
    );
    assert_eq!(info.policy, None);
}
#[test]
fn cannot_create_same_vid() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            // NOTE: we cannot test feegrant on osmosis-test-tube as it is not registered
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid: vid.clone(),
            initial_data: vec![],
            plugins: vec![],
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, DENOM)],
            &suite.accounts[IDEPLOYER],
        )
        .unwrap();

    // Recreate again
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, DENOM)],
            &suite.accounts[IDEPLOYER],
        )
        .unwrap_err();
}
