use crate::{
    constants::*,
    helpers::default_entity,
    test_tube::{test_env::HubChainSuite, util::contract::Contract},
};
use cosmwasm_std::{coin, to_binary, Addr, Binary};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Bank, OsmosisTestApp};
use test_tube::module::Module;
use vectis_wallet::{
    interface::{
        factory_management_trait::sv::QueryMsg as FactoryMgmtQueryMsg,
        factory_service_trait::sv::{
            ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
        },
        wallet_trait::sv::QueryMsg as WalletQueryMsg,
    },
    types::{
        factory::CreateWalletMsg,
        wallet::{WalletAddrs, WalletInfo},
    },
};
// NOTE: we cannot test feegrant on osmosis-test-tube as it is not registered
// we also cannot test create ica - pending neutron test tube

// We cannot test with relayer in osmosis-test-tube.
// Pending neutorn test-tube
#[test]
fn create_wallet_successfully_without_relayer() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let chains = (NON_IBC_CHAIN_NAME.into(), NON_IBC_CHAIN_ADDR.into());
    let initial_data = (
        to_binary("some-key").unwrap(),
        to_binary("some-value").unwrap(),
    );

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid: vid.clone(),
            initial_data: vec![initial_data.clone()],
            plugins: vec![],
            code_id: None,
            chains: Some(vec![chains]),
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    let res = factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, "uosmo")],
            &suite.accounts[IRELAYER],
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
        .any(|e| e.ty == "wasm-vectis.factory.v1"));

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
                addr: Some(NON_IBC_CHAIN_ADDR.into())
            },
        ]
    );
    assert_eq!(info.policy, None);
}

#[test]
fn create_with_inital_balance() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![coin(INIT_BALANCE, DENOM)],
            vid: vid.clone(),
            initial_data: vec![],
            plugins: vec![],
            code_id: None,
            chains: None,
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);

    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE + INIT_BALANCE, DENOM)],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid })
        .unwrap();

    let bank = Bank::new(&app);
    let init_balance = bank
        .query_balance(&QueryBalanceRequest {
            address: wallet_addr.unwrap().to_string(),
            denom: DENOM.into(),
        })
        .unwrap();
    assert_eq!(
        init_balance.balance.unwrap().amount,
        INIT_BALANCE.to_string()
    );
}

//#[test]
//fn create_with_ica() {
//    let app = OsmosisTestApp::new();
//    let suite = HubChainSuite::init(&app);
//
//    let entity = default_entity();
//    let vid = String::from("user-name@vectis");
//
//    let create_msg = FactoryServiceExecMsg::CreateWallet {
//        create_wallet_msg: CreateWalletMsg {
//            controller: entity.clone(),
//            relayers: vec![],
//            proxy_initial_funds: vec![coin(INIT_BALANCE, DENOM)],
//            vid: vid.clone(),
//            initial_data: vec![],
//            plugins: vec![],
//            code_id: None,
//            chains: Some(vec![(
//                IBC_CHAIN_NAME.into(),
//                r#"{"version":"ics27-1","encoding":"proto3","tx_type":"sdk_multi_msg","controller_connection_id":"connection-1","host_connection_id":"connection-12"}"#.into(),
//            )]),
//        },
//    };
//
//    let factory = Contract::from_addr(&app, suite.factory);
//
//    factory
//        .execute(
//            &create_msg,
//            &[coin(WALLET_FEE + INIT_BALANCE, DENOM)],
//            &suite.accounts[IDEPLOYER],
//        )
//        .unwrap();
//
//    let wallet_addr: Option<Addr> = factory
//        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid.into() })
//        .unwrap();
//    let wallet_addr = wallet_addr.unwrap().to_string();
//
//    let bank = Bank::new(&app);
//    let init_balance = bank
//        .query_balance(&QueryBalanceRequest {
//            address: wallet_addr.clone(),
//            denom: DENOM.into(),
//        })
//        .unwrap();
//    assert_eq!(
//        init_balance.balance.unwrap().amount,
//        INIT_BALANCE.to_string()
//    );
//
//    let wallet = Contract::from_addr(&app, wallet_addr);
//
//    let info: WalletInfo = wallet.query(&wallet_trait::QueryMsg::Info {}).unwrap();
//
//    println!("info {:?}", info);
//}

#[test]
fn cannot_create_with_incorrect_total_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![coin(INIT_BALANCE, DENOM)],
            vid: vid.clone(),
            initial_data: vec![],
            plugins: vec![],
            code_id: None,
            chains: None,
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    // incorrect fee: should be INIT_BALANCE + WALLET_FEE
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, DENOM)],
            &suite.accounts[IRELAYER],
        )
        .unwrap_err();
}

#[test]
fn cannot_create_using_same_vid() {
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
            code_id: None,
            chains: None,
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, DENOM)],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    // Recreate again
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, DENOM)],
            &suite.accounts[IRELAYER],
        )
        .unwrap_err();
}

#[test]
fn cannot_create_with_incorrect_fee() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid: vid.clone(),
            initial_data: vec![],
            plugins: vec![],
            code_id: None,
            chains: None,
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    // incorrect wallet fee
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE - 3u128, DENOM)],
            &suite.accounts[IRELAYER],
        )
        .unwrap_err();
}

#[test]
fn create_wallet_with_addrs_can_be_queired() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let entity = default_entity();
    let vid = String::from("user-name@vectis");

    let chains = (NON_IBC_CHAIN_NAME.into(), NON_IBC_CHAIN_ADDR.into());

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![],
            vid: vid.clone(),
            initial_data: vec![],
            plugins: vec![],
            code_id: None,
            chains: Some(vec![chains]),
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE, "uosmo")],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid.clone() })
        .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.unwrap().to_string());

    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    assert_eq!(
        info.addresses,
        vec![
            WalletAddrs {
                chain_id: IBC_CHAIN_NAME.into(),
                addr: None
            },
            WalletAddrs {
                chain_id: NON_IBC_CHAIN_NAME.into(),
                addr: Some(NON_IBC_CHAIN_ADDR.into())
            },
        ]
    );

    let wallet_addr_remote: Option<String> = factory
        .query(&FactoryServiceQueryMsg::WalletByVidChain {
            vid: vid.clone(),
            chain_id: NON_IBC_CHAIN_NAME.into(),
        })
        .unwrap();

    assert_eq!(Some(NON_IBC_CHAIN_ADDR.to_string()), wallet_addr_remote)
}
