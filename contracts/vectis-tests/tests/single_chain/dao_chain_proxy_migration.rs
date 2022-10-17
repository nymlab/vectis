use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, WasmMsg};
use cw_multi_test::Executor;
use vectis_factory::{msg::ExecuteMsg as FactoryExecuteMsg, ContractError};
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_wallet::{ProxyMigrateMsg, ProxyMigrationTxMsg, WalletAddr, WalletInfo};

use crate::common::*;

#[test]
fn user_can_migrate_proxy_with_direct_message() {
    let mut suite = DaoChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![init_proxy_fund.clone()],
        None,
        WALLET_FEE + init_proxy_fund.amount.u128(),
    );
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;
    let old_code_id = w.code_id;
    assert_eq!(old_code_id, suite.sc_proxy_id);

    let new_code_id = suite.app.store_code(contract_proxy());
    suite
        .update_proxy_code_id(new_code_id, suite.factory_addr.clone())
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
        suite.factory_addr.clone(),
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
        to_address: suite.factory_addr.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        user,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute { msgs: vec![msg] },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let wallet_fund = suite
        .query_balance(&wallet_address, "ucosm".into())
        .unwrap();

    assert_eq!(
        init_proxy_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn relayer_can_migrate_proxy_with_user_signature() {
    let mut suite = DaoChainSuite::init().unwrap();
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let old_code_id = w.code_id;
    let relayer = w.relayers.pop().unwrap();
    assert_eq!(old_code_id, suite.sc_proxy_id);

    let new_code_id = suite.app.store_code(contract_proxy());
    let r = suite.update_proxy_code_id(new_code_id, suite.factory_addr.clone());
    assert!(r.is_ok());

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id,
        msg: to_binary(&ProxyMigrateMsg { new_code_id }).unwrap(),
    });

    let relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        suite.factory_addr.clone(),
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );

    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

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
        .execute_contract(
            not_user,
            suite.factory_addr.clone(),
            &migrate_wallet_msg,
            &[],
        )
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

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
        .execute_contract(
            w.user_addr,
            suite.factory_addr.clone(),
            &migrate_wallet_msg,
            &[],
        )
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );
    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        migration_msg: ProxyMigrationTxMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::ClearAdmin {
                contract_addr: String::from("randomaddr"),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            w.user_addr,
            suite.factory_addr.clone(),
            &migrate_wallet_msg,
            &[],
        )
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

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
            suite.factory_addr.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address.clone()),
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

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
            suite.factory_addr.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address.clone()),
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
    let create_proxy_rsp = suite.create_new_proxy(
        Addr::unchecked(USER_ADDR),
        suite.factory_addr.clone(),
        vec![],
        None,
        WALLET_FEE,
    );

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_unclaimed_govec_wallets(&suite.factory_addr, None, None)
        .unwrap()
        .wallets
        .pop()
        .unwrap()
        .0;

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
            suite.factory_addr.clone(),
            &FactoryExecuteMsg::MigrateWallet {
                wallet_address: WalletAddr::Addr(wallet_address.clone()),
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
