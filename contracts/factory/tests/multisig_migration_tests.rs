use cosmwasm_std::{coin, to_binary, Addr, Coin, CosmosMsg, WasmMsg};
use cw3::VoterDetail;
use cw_multi_test::Executor;
use sc_wallet::{Guardians, MultiSig, WalletInfo};
use wallet_proxy::msg::ExecuteMsg as ProxyExecuteMsg;

pub mod common;
use common::*;

#[test]
fn user_can_migrate_proxy_multisig_with_direct_message() {
    let mut suite = Suite::init().unwrap();
    let init_wallet_fund: Coin = coin(400, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![init_wallet_fund],
    );
    let init_proxy_fund: Coin = coin(300, "ucosm");
    let init_multisig_fund: Coin = coin(50, "ucosm");

    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![init_multisig_fund],
    };

    let create_proxy_rsp = suite.create_new_proxy(
        factory.clone(),
        vec![init_proxy_fund.clone()],
        Some(multisig.clone()),
    );

    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    let user = w.user_addr;

    let old_multisig_addr = w.multisig_address;

    let new_multisig_code_id = suite.app.store_code(contract_multisig());
    let r = suite.update_proxy_multisig_code_id(new_multisig_code_id, factory.clone());
    assert!(r.is_ok());

    // User update their proxy related multisig contract to the new guardian set
    // This reinstantiates a new contract and changes the stored multisig contract addr
    let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {
        guardians: Guardians {
            addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
            guardians_multisig: Some(multisig),
        },
        new_multisig_code_id: None,
    };

    let execute_msg_resp = suite.app.execute_contract(
        user.clone(),
        wallet_address.clone(),
        &update_guardians_message,
        &[],
    );

    assert!(execute_msg_resp.is_ok());
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_ne!(new_w.multisig_address, old_multisig_addr);
    assert!(!new_w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(new_w.guardians.contains(&Addr::unchecked(GUARD3)));

    let new_multisig_voters = suite
        .query_multisig_voters(&new_w.multisig_address.unwrap())
        .unwrap();
    assert!(new_multisig_voters.voters.contains(&VoterDetail {
        addr: GUARD3.to_string(),
        weight: 1
    }));
    assert!(new_multisig_voters.voters.contains(&VoterDetail {
        addr: GUARD2.to_string(),
        weight: 1
    }));
    assert!(!new_multisig_voters.voters.contains(&VoterDetail {
        addr: GUARD1.to_string(),
        weight: 1
    }));
}

#[test]
fn relayer_can_migrate_proxy_multisig_with_user_signature() {
    let mut suite = Suite::init().unwrap();
    let init_wallet_fund: Coin = coin(400, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![init_wallet_fund],
    );
    let init_proxy_fund: Coin = coin(300, "ucosm");
    let init_multisig_fund: Coin = coin(50, "ucosm");

    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![init_multisig_fund],
    };

    let create_proxy_rsp = suite.create_new_proxy(
        factory.clone(),
        vec![init_proxy_fund.clone()],
        Some(multisig.clone()),
    );

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    let old_multisig_code_id = w.multisig_code_id;

    let relayer = w.relayers.pop().unwrap();
    assert_eq!(old_multisig_code_id, suite.sc_proxy_multisig_code_id);

    let new_multisig_code_id = suite.app.store_code(contract_multisig());
    let r = suite.update_proxy_multisig_code_id(new_multisig_code_id, factory.clone());
    assert!(r.is_ok());

    let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {
        guardians: Guardians {
            addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
            guardians_multisig: Some(multisig),
        },
        new_multisig_code_id: Some(new_multisig_code_id),
    };

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

    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.multisig_code_id, new_multisig_code_id);
    assert_ne!(new_multisig_code_id, old_multisig_code_id);
}
