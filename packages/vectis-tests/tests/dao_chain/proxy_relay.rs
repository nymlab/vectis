use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, WasmMsg};
use cw_multi_test::Executor;
use vectis_factory::{msg::ExecuteMsg as FactoryExecuteMsg, ContractError};
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_wallet::{ProxyMigrateMsg, ProxyMigrationTxMsg, WalletAddr, WalletInfo};

use crate::common::dao_common::*;

#[test]
fn relay_proxy_user_tx_succeeds() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let relay_transaction = suite.create_relay_transaction(
        USER_PRIV,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: wallet_address.to_string(),
            msg: to_binary(&update_guardians_message).unwrap(),
            funds: vec![],
        }),
        w.nonce,
    );

    let relay_msg: ProxyExecuteMsg = ProxyExecuteMsg::Relay {
        transaction: relay_transaction,
    };

    let execute_msg_resp =
        suite
            .app
            .execute_contract(relayer, wallet_address.clone(), &relay_msg, &[]);
    assert!(execute_msg_resp.is_ok());

    let coins = coins(2, "token");

    let mut deps = mock_dependencies_with_balance(&coins);
    do_instantiate(deps.as_mut());

    // RELAYER1 is a valid relayer
    let info = mock_info(RELAYER1, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bank_msg = BankMsg::Burn { amount: coins };
    let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);
    let msg_bytes = to_binary(&cosmos_msg).unwrap();
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_bytes
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_bytes.to_vec()),
        user_pubkey: Binary(public_key.serialize().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce,
    };

    let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap();

    assert_eq!(response.attributes, [("action", "execute_relay")]);
}

#[test]
fn relay_proxy_user_tx_invalid_msg_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // RELAYER1 is a valid relayer
    let info = mock_info(RELAYER1, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let msg_slice = [0xab; 32];
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_slice
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_slice.to_vec()),
        user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce,
    };

    let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

    match response {
        ContractError::InvalidMessage { msg: _ } => {}
        _ => panic!("Not correct response"),
    }
}

#[test]
fn relay_proxy_user_tx_is_not_relayer_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let coins = coins(2, "token");

    let mut deps = mock_dependencies_with_balance(&coins);
    do_instantiate(deps.as_mut());

    // INVALID_RELAYER is not a valid relayer
    let info = mock_info(INVALID_RELAYER, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bank_msg = BankMsg::Burn { amount: coins };
    let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);
    let msg_bytes = to_binary(&cosmos_msg).unwrap();
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_bytes
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_bytes.to_vec()),
        user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce,
    };

    let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

    assert_eq!(response, ContractError::IsNotRelayer {});
}

#[test]
fn relay_proxy_user_tx_is_not_user_fails() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let coins = coins(2, "token");

    let mut deps = mock_dependencies_with_balance(&coins);
    do_instantiate(deps.as_mut());

    // RELAYER1 is a valid relayer
    let info = mock_info(RELAYER1, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    // Make an attempt to relay message of nonexistent user
    let secp = Secp256k1::new();
    let secret_key =
        SecretKey::from_slice(INVAILID_USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bank_msg = BankMsg::Burn { amount: coins };
    let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);
    let msg_bytes = to_binary(&cosmos_msg).unwrap();
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_bytes
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_bytes.to_vec()),
        user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce,
    };

    let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

    assert_eq!(response, ContractError::IsNotUser {});
}

#[test]
fn relay_proxy_user_tx_invalid_nonce_fails() {
    let coins = coins(2, "token");

    let mut deps = mock_dependencies_with_balance(&coins);
    do_instantiate(deps.as_mut());

    // RELAYER1 is a valid relayer
    let info = mock_info(RELAYER1, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bank_msg = BankMsg::Burn { amount: coins };

    let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);

    let msg_bytes = to_binary(&cosmos_msg).unwrap();

    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_bytes
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_bytes.to_vec()),
        user_pubkey: Binary(public_key.serialize().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce: nonce + 1,
    };

    let response = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();

    assert_eq!(
        response,
        ContractError::RelayTxError(RelayTxError::NoncesAreNotEqual {})
    );
}

#[test]
fn frozen_contract_relay_proxy_user_tx_fails() {
    let coins = coins(2, "token");

    let mut deps = mock_dependencies_with_balance(&coins);
    do_instantiate(deps.as_mut());

    // GUARD1 is a valid relayer
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(response.attributes, [("action", "frozen")]);

    // RELAYER1 is a valid relayer
    let info = mock_info(RELAYER1, &[]);
    let nonce = query_info(deps.as_ref()).unwrap().nonce;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bank_msg = BankMsg::Burn { amount: coins };
    let cosmos_msg = CosmosMsg::<()>::Bank(bank_msg);
    let msg_bytes = to_binary(&cosmos_msg).unwrap();
    let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
        &msg_bytes
            .iter()
            .chain(&nonce.to_be_bytes())
            .copied()
            .collect::<Vec<u8>>(),
    );
    let sig = secp.sign(&message_with_nonce, &secret_key);

    let relay_transaction = RelayTransaction {
        message: Binary(msg_bytes.to_vec()),
        user_pubkey: Binary(public_key.serialize().to_vec()),
        signature: Binary(sig.serialize_compact().to_vec()),
        nonce,
    };

    let err = execute_relay(deps.as_mut(), info, relay_transaction).unwrap_err();
    assert_eq!(err, ContractError::Frozen {});
}

#[test]
fn user_can_migrate_proxy_with_direct_message() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let wallet_address = suite
        .create_new_proxy(
            suite.user.clone(),
            vec![init_proxy_fund.clone()],
            None,
            WALLET_FEE + init_proxy_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;
    let old_code_id = w.code_id;

    let new_code_id = suite.app.store_code(contract_proxy());
    suite
        .update_proxy_code_id(new_code_id, suite.factory.clone())
        .unwrap();

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        migration_msg: ProxyMigrationTxMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id,
                msg: to_binary(&ProxyMigrateMsg { new_code_id }).unwrap(),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_resp = suite.app.execute_contract(
        user.clone(),
        suite.factory.clone(),
        &migrate_wallet_msg,
        &[],
    );

    assert!(execute_msg_resp.is_ok());
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, new_code_id);
    assert_ne!(new_code_id, old_code_id);

    // user can execute message after migration
    let send_amount: Coin = coin(10, "ucosm");
    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: suite.factory.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        user,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute { msgs: vec![msg] },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let wallet_fund = suite.query_balance(&wallet_address).unwrap();

    assert_eq!(
        init_proxy_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn relayer_can_migrate_proxy_with_user_signature() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let old_code_id = w.code_id;
    let relayer = w.relayers.pop().unwrap();

    let new_code_id = suite.app.store_code(contract_proxy());
    let r = suite.update_proxy_code_id(new_code_id, suite.factory.clone());
    assert!(r.is_ok());

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id,
        msg: to_binary(&ProxyMigrateMsg { new_code_id }).unwrap(),
    });

    let relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        suite.factory.clone(),
        &FactoryExecuteMsg::MigrateWallet {
            wallet_address: WalletAddr::Addr(wallet_address.clone()),
            migration_msg: ProxyMigrationTxMsg::RelayTx(relay_transaction),
        },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, new_code_id);
    assert_ne!(new_code_id, old_code_id);
}

#[test]
fn user_cannot_migrate_others_wallet() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(suite.user.clone(), vec![], None, WALLET_FEE)
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let code_id = w.code_id;

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        migration_msg: ProxyMigrationTxMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id: code_id,
                msg: to_binary(&ProxyMigrateMsg {
                    new_code_id: code_id,
                })
                .unwrap(),
            }))
            .unwrap(),
        ),
    };

    let not_user = Addr::unchecked("not_user");

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(not_user, suite.factory.clone(), &migrate_wallet_msg, &[])
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(execute_msg_err.to_string(), String::from("Unauthorized"));
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, code_id);
}

#[test]
fn user_cannot_migrate_with_mismatched_code_id() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let code_id = w.code_id;

    let unsupported_code_id = suite.app.store_code(contract_proxy());
    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        migration_msg: ProxyMigrationTxMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id: unsupported_code_id,
                msg: to_binary(&ProxyMigrateMsg {
                    new_code_id: unsupported_code_id,
                })
                .unwrap(),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(w.user_addr, suite.factory.clone(), &migrate_wallet_msg, &[])
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        execute_msg_err.to_string(),
        String::from("InvalidMigrationMsg: MismatchProxyCodeId")
    );
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, code_id);
}

#[test]
fn user_cannot_migrate_with_invalid_wasm_msg() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address),
        migration_msg: ProxyMigrationTxMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::ClearAdmin {
                contract_addr: String::from("randomaddr"),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(w.user_addr, suite.factory.clone(), &migrate_wallet_msg, &[])
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(
        execute_msg_err.to_string(),
        String::from("InvalidMigrationMsg: InvalidWasmMsg")
    );
}

#[test]
fn relayer_cannot_migrate_others_wallet() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&ProxyMigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce + 123);

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            relayer,
            suite.factory.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address),
                migration_msg: ProxyMigrationTxMsg::RelayTx(relay_transaction),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        execute_msg_err.to_string(),
        String::from("InvalidRelayMigrationTx: NoncesAreNotEqual")
    );
}

#[test]
fn relayer_cannot_migrate_proxy_with_mismatch_user_addr() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&ProxyMigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let mut relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    // invalid user_pubkey
    relay_transaction.user_pubkey = Binary([0; 33].to_vec());

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            relayer,
            suite.factory.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address),
                migration_msg: ProxyMigrationTxMsg::RelayTx(relay_transaction),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        execute_msg_err.to_string(),
        String::from("InvalidRelayMigrationTx: MismatchUserAddr")
    );
}

#[test]
fn relayer_cannot_migrate_proxy_with_invalid_signature() {
    let mut suite = DaoChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&ProxyMigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let mut relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    // invalid signature
    relay_transaction.signature = Binary(
        [
            1, 210, 8, 128, 147, 77, 89, 146, 29, 147, 127, 232, 221, 182, 94, 13, 73, 114, 228,
            48, 12, 21, 115, 63, 52, 224, 231, 92, 110, 8, 80, 30, 17, 93, 50, 211, 114, 25, 194,
            139, 64, 172, 4, 135, 99, 63, 178, 84, 1, 138, 204, 203, 229, 83, 249, 167, 42, 106,
            33, 109, 1, 1, 1, 1,
        ]
        .to_vec(),
    );

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            relayer,
            suite.factory.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address),
                migration_msg: ProxyMigrationTxMsg::RelayTx(relay_transaction),
            },
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(
        execute_msg_err.to_string(),
        String::from("InvalidRelayMigrationTx: SignatureVerificationError")
    );
}
