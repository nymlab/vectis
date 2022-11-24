use cosmwasm_std::{coin, Addr, BankMsg, Binary, Coin, CosmosMsg};
use cw_multi_test::Executor;
use vectis_proxy::{msg::ExecuteMsg as ProxyExecuteMsg, ContractError};
use vectis_wallet::WalletInfo;

use crate::common::{common::*, dao_common::*};

#[test]
fn relay_proxy_controller_tx_succeeds() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    let relay_transaction = suite.create_relay_transaction(
        CONTROLLER_PRIV,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "SomeAddr".to_string(),
            amount: vec![coin(1, "ucosm")],
        }),
        w.nonce,
    );

    let relay_msg: ProxyExecuteMsg = ProxyExecuteMsg::Relay {
        transaction: relay_transaction,
    };

    let relayer = String::from("relayer");
    let execute_msg_resp = suite.app.execute_contract(
        Addr::unchecked(relayer),
        wallet_address.clone(),
        &relay_msg,
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    assert_eq!(
        suite.query_balance(&wallet_address).unwrap().amount,
        Uint128::from(89u16)
    )
}

#[test]
fn relay_proxy_controller_tx_invalid_msg_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(CONTROLLER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let msg_slice = [0xab; 32];
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_slice
            .iter()
            .chain(&w.nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_slice.to_vec()),
        controller_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce: w.nonce,
    };
    let relay_msg: ProxyExecuteMsg = ProxyExecuteMsg::Relay {
        transaction: relay_transaction,
    };

    let relayer = String::from("relayer");
    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            Addr::unchecked(relayer),
            wallet_address.clone(),
            &relay_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    match execute_msg_err {
        ContractError::InvalidMessage { msg: _msg } => {}
        _ => assert!(false),
    }
}

#[test]
fn relay_proxy_controller_tx_invalid_relayer_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    let relay_transaction = suite.create_relay_transaction(
        CONTROLLER_PRIV,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "SomeAddr".to_string(),
            amount: vec![coin(1, "ucosm")],
        }),
        w.nonce,
    );

    let relay_msg: ProxyExecuteMsg = ProxyExecuteMsg::Relay {
        transaction: relay_transaction,
    };

    let relayer = String::from("invalid_relayer");
    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            Addr::unchecked(relayer),
            wallet_address.clone(),
            &relay_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(execute_msg_err, ContractError::IsNotRelayer {});
}

#[test]
fn relay_proxy_controller_tx_is_not_controller_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");

    // Creates a wallet held by the DAO
    let wallet_address = suite
        .create_new_proxy(
            suite.controller.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    // Controller NOT DAO signs
    let relay_transaction = suite.create_relay_transaction(
        NON_CONTROLLER_PRIV,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "SomeAddr".to_string(),
            amount: vec![coin(1, "ucosm")],
        }),
        w.nonce,
    );

    let relay_msg: ProxyExecuteMsg = ProxyExecuteMsg::Relay {
        transaction: relay_transaction,
    };

    let relayer = String::from("relayer");
    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            Addr::unchecked(relayer),
            wallet_address.clone(),
            &relay_msg,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(execute_msg_err, ContractError::IsNotController {});
}
