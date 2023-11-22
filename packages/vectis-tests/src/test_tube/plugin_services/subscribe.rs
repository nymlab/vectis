use crate::{
    constants::*,
    helpers::sign_and_create_relay_tx,
    test_tube::{
        test_env::HubChainSuite,
        util::{contract::Contract, wallet::create_webauthn_wallet},
    },
};
use cosmwasm_std::{coin, to_binary, CosmosMsg, WasmMsg};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Bank, OsmosisTestApp};
use serial_test::serial;
use test_tube::module::Module;
use vectis_wallet::{
    interface::registry_service_trait::sv::{
        ExecMsg as RegistryServiceExecMsg, QueryMsg as RegistryServiceQueryMsg,
    },
    interface::wallet_trait::sv::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    types::plugin_registry::{Subscriber, SubscriptionTier},
    types::wallet::WalletInfo,
};

#[test]
#[serial]
fn proxy_can_subscribe_with_fee() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);
    let vid = "test-user";
    suite.register_plugins();

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let bank = Bank::new(&app);
    let registry = Contract::from_addr(&app, suite.plugin_registry.clone());
    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let init_balance_deployer = bank
        .query_balance(&QueryBalanceRequest {
            address: suite.deployer.clone(),
            denom: DENOM.into(),
        })
        .unwrap();

    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: suite.plugin_registry,
            msg: to_binary(&RegistryServiceExecMsg::subscribe(SubscriptionTier::L1)).unwrap(),
            funds: vec![coin(TIER_1_FEE, DENOM)],
        })],
        info.controller.nonce,
        vid,
    );

    wallet
        .execute(
            &WalletExecMsg::AuthExec {
                transaction: relay_tx,
            },
            &[],
            &suite.accounts[IRELAYER],
        )
        .unwrap();

    let post_balance_deployer = bank
        .query_balance(&QueryBalanceRequest {
            address: suite.deployer.clone(),
            denom: DENOM.into(),
        })
        .unwrap();

    // deployer has the fee as expected
    let init_balance =
        u128::from_str_radix(&init_balance_deployer.balance.unwrap().amount, 10).unwrap();
    assert_eq!(
        (init_balance + TIER_1_FEE).to_string(),
        post_balance_deployer.balance.unwrap().amount
    );

    // registry records updates
    let subscriber: Option<Subscriber> = registry
        .query(&RegistryServiceQueryMsg::SubsciptionDetails {
            addr: wallet_addr.to_string(),
        })
        .unwrap();
    assert_eq!(subscriber.unwrap().tier, SubscriptionTier::L1);
}

#[test]
#[serial]
fn proxy_cannot_subscribe_without_correct_fee() {
    let app = OsmosisTestApp::new();
    let mut suite = HubChainSuite::init(&app);
    let vid = "test-user";
    suite.register_plugins();

    let (wallet_addr, _) = create_webauthn_wallet(
        &app,
        &suite.factory,
        vid,
        INIT_BALANCE,
        &suite.accounts[IRELAYER],
    );

    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();
    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: suite.plugin_registry,
            msg: to_binary(&RegistryServiceExecMsg::subscribe(SubscriptionTier::L1)).unwrap(),
            // Tier 1 fees not provided
            funds: vec![],
        })],
        info.controller.nonce,
        vid,
    );

    wallet
        .execute(
            &WalletExecMsg::AuthExec {
                transaction: relay_tx,
            },
            &[],
            &suite.accounts[IRELAYER],
        )
        .unwrap_err();
}
