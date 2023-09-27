use cosmwasm_std::{to_binary, Binary, CosmosMsg, Empty};
use osmosis_test_tube::OsmosisTestApp;
use serial_test::serial;

use vectis_wallet::interface::wallet_trait;


use crate::{
    constants::*,
    test_tube::{
        test_env::HubChainSuite,
        util::{
            contract::Contract,
            wallet::{create_webauthn_wallet, sign_and_submit},
        },
    },
};

#[test]
#[serial]
fn controller_can_update_data() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);
    let vid = "test-user";

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let data = vec![
        (
            to_binary("some_key").unwrap(),
            Some(to_binary("some-value").unwrap()),
        ),
        (
            to_binary("some_key_1").unwrap(),
            Some(to_binary("some-value-1").unwrap()),
        ),
    ];

    let update_data_msg = wallet_trait::ExecMsg::UpdateData { data: data.clone() };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&update_data_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());

    let mut value: Option<Binary> = wallet
        .query(&wallet_trait::QueryMsg::Data {
            key: data[0].clone().0,
        })
        .unwrap();

    assert_eq!(value.unwrap(), data[0].clone().1.unwrap());

    value = wallet
        .query(&wallet_trait::QueryMsg::Data {
            key: data[1].clone().0,
        })
        .unwrap();

    assert_eq!(value.unwrap(), data[1].clone().1.unwrap());

    let data = vec![(to_binary("some_key_1").unwrap(), None)];

    let update_data_msg = wallet_trait::ExecMsg::UpdateData { data: data.clone() };

    sign_and_submit(
        &app,
        vec![CosmosMsg::<Empty>::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&update_data_msg).unwrap(),
            funds: vec![],
        })],
        vid,
        wallet_addr.as_str(),
        &suite.accounts[IRELAYER],
    )
    .unwrap();

    value = wallet
        .query(&wallet_trait::QueryMsg::Data {
            key: data[0].clone().0,
        })
        .unwrap();

    assert!(value.is_none());
}
