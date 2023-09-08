use super::util::passkey::{
    de_client_data, hash_to_base64url_string, hash_to_hex_string, must_create_credential,
    must_get_credential,
};
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Empty};
use osmosis_std::types::cosmos::bank::v1beta1::QueryBalanceRequest;
use osmosis_test_tube::{Account, Bank, OsmosisTestApp, SigningAccount};
use serde_json_wasm;
use test_tube::module::Module;

use vectis_wallet::{
    interface::{
        factory_management_trait::QueryMsg as FactoryMgmtQueryMsg,
        factory_service_trait::{
            ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
        },
        wallet_trait::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    },
    types::{
        authenticator::AuthenticatorType,
        entity::Entity,
        factory::CreateWalletMsg,
        wallet::{
            RelayTransaction, VectisRelayedTx, WalletAddrs, WalletInfo, WebauthnRelayedTxMsg,
        },
    },
};

use super::{
    test_env::HubChainSuite,
    util::{
        constants::*,
        contract::Contract,
        wallet::{default_entity, webauthn_entity},
    },
};

#[test]
fn wallet_can_do_webauthn_tx() {
    let app = OsmosisTestApp::new();
    let suite = HubChainSuite::init(&app);

    let vid1 = "test-user";
    let pubkey = must_create_credential(vid1);

    let entity = webauthn_entity(&pubkey);
    let transfer = coin(5, DENOM);

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![coin(INIT_BALANCE, DENOM)],
            vid: vid1.into(),
            initial_data: vec![],
            plugins: vec![],
        },
    };

    let factory = Contract::from_addr(&app, suite.factory);
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE + INIT_BALANCE, DENOM)],
            &suite.accounts[IDEPLOYER],
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid1.into() })
        .unwrap();

    let wallet = Contract::from_addr(&app, wallet_addr.unwrap().to_string());

    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();
    assert_eq!(info.controller.nonce, 0);
    assert_eq!(info.controller.data, to_binary(&pubkey).unwrap());

    let init_balance = Bank::new(&app)
        .query_balance(&QueryBalanceRequest {
            address: wallet.contract_addr.clone(),
            denom: DENOM.into(),
        })
        .unwrap();

    // =======================
    // Signing data
    // =======================
    let signed_msg = VectisRelayedTx {
        messages: vec![CosmosMsg::<Empty>::Bank(BankMsg::Send {
            to_address: suite.accounts[IDEPLOYER].address(),
            amount: vec![transfer],
        })],
        nonce: info.controller.nonce,
        sponsor_fee: None,
    };

    let signed_data = serde_json_wasm::to_string(&signed_msg).unwrap();

    // We encode the hash vec into hex to pass to the cli
    let challenge = hash_to_hex_string(signed_data.as_bytes());
    let response = must_get_credential(vid1, challenge.clone());

    // Compare the expeted challenge value to the one returned from passkey
    let expect_challenge = hash_to_base64url_string(signed_data.as_bytes());
    let challenge_in_client_data = de_client_data(&response.client_data_json);
    assert_eq!(expect_challenge, challenge_in_client_data.challenge);

    let relay_tx = RelayTransaction {
        message: to_binary(&WebauthnRelayedTxMsg {
            signed_data,
            auth_data: to_binary(&response.authenticator_data.to_owned()).unwrap(),
            client_data: to_binary(&response.client_data_json.to_owned()).unwrap(),
        })
        .unwrap(),
        signature: to_binary(&response.signature.to_owned()).unwrap(),
    };

    let res = wallet.execute(
        &WalletExecMsg::AuthExec {
            transaction: relay_tx,
        },
        &[],
        &suite.accounts[IRELAYER],
    );

    println!("RES: {:?}", res);
}
