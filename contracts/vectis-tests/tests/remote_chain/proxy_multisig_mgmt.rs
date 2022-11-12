use cosmwasm_std::{coin, to_binary, Addr, Coin, CosmosMsg, WasmMsg};
use cw3::VoterDetail;
use cw_multi_test::Executor;
use vectis_proxy::msg::ExecuteMsg as ProxyExecuteMsg;
use vectis_proxy::ContractError;
use vectis_wallet::{Guardians, GuardiansUpdateMsg, MultiSig, WalletInfo};

use crate::common::common::*;
use crate::common::remote_common::*;

#[test]
fn user_can_update_proxy_multisig_with_direct_message() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let init_proxy_fund: Coin = coin(300, "uremote");
    let init_multisig_fund: Coin = coin(50, "uremote");

    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![init_multisig_fund.clone()],
    };

    let wallet_address = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            vec![init_proxy_fund.clone()],
            Some(multisig.clone()),
            WALLET_FEE + init_proxy_fund.amount.u128() + init_multisig_fund.amount.u128(),
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;
    let old_multisig_addr = w.multisig_address;

    let new_guardians = Guardians {
        addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
        guardians_multisig: Some(multisig),
    };

    // User update their proxy related multisig contract to the new guardian set
    // This reinstantiates a new contract and changes the stored multisig contract addr
    suite
        .create_guardians_request_and_update_guardians(&user, &wallet_address, new_guardians, None)
        .unwrap();

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
fn proxy_without_multisig_can_instantiate_new_multisig_guardian() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(w.multisig_address, None);

    let guardians = Guardians {
        addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
        guardians_multisig: Some(MultiSig {
            threshold_absolute_count: MULTISIG_THRESHOLD,
            multisig_initial_funds: vec![],
        }),
    };

    // User update their proxy guardian to multisig with factory stored code id
    suite
        .create_guardians_request_and_update_guardians(
            &Addr::unchecked(USER_ADDR),
            &wallet_address,
            guardians,
            None,
        )
        .unwrap();

    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert!(new_w.multisig_address.is_some());
    assert!(!new_w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(new_w.guardians.contains(&Addr::unchecked(GUARD3)));
}

#[test]
fn user_can_remove_multisig_for_guardians() {
    let mut suite = RemoteChainSuite::init().unwrap();

    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![],
    };

    let wallet_address = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            vec![],
            Some(multisig),
            WALLET_FEE,
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert!(w.multisig_address.is_some());

    let guardians = Guardians {
        addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };

    suite
        .create_guardians_request_and_update_guardians(
            &Addr::unchecked(USER_ADDR),
            &wallet_address,
            guardians,
            None,
        )
        .unwrap();

    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.multisig_address, None);
}

#[test]
fn relayer_can_update_proxy_multisig_with_user_signature() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![],
    };

    let wallet_address = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            vec![],
            Some(multisig.clone()),
            WALLET_FEE,
        )
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    let old_multisig_code_id = w.multisig_code_id;

    let relayer = w.relayers.pop().unwrap();

    let new_multisig_code_id = suite.app.store_code(contract_multisig());
    assert_ne!(old_multisig_code_id, new_multisig_code_id);
    let r = suite.update_proxy_multisig_code_id(new_multisig_code_id, suite.factory.clone());
    assert!(r.is_ok());

    let new_guardians = Guardians {
        addresses: vec![GUARD2.to_string(), GUARD3.to_string()],
        guardians_multisig: Some(multisig),
    };

    let request = GuardiansUpdateMsg {
        guardians: new_guardians,
        new_multisig_code_id: Some(new_multisig_code_id),
    };

    let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::RequestUpdateGuardians {
        request: Some(request),
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

    let query_request = suite
        .query_guardians_request(&wallet_address)
        .unwrap()
        .unwrap();
    assert_eq!(
        query_request.new_multisig_code_id.unwrap(),
        new_multisig_code_id
    );
}

#[test]
fn non_user_update_proxy_multisig_fails() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(
            Addr::unchecked(USER_ADDR),
            vec![],
            Some(MultiSig {
                threshold_absolute_count: MULTISIG_THRESHOLD,
                multisig_initial_funds: vec![],
            }),
            WALLET_FEE,
        )
        .unwrap();

    let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {};

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(
            Addr::unchecked("not-user"),
            wallet_address,
            &update_guardians_message,
            &[],
        )
        .unwrap_err()
        .downcast()
        .unwrap();
    assert_eq!(execute_msg_err, ContractError::IsNotUser {})
}

#[test]
fn relayer_update_proxy_multisig_with_non_user_fails() {
    let mut suite = RemoteChainSuite::init().unwrap();
    let wallet_address = suite
        .create_new_proxy(Addr::unchecked(USER_ADDR), vec![], None, WALLET_FEE)
        .unwrap();

    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {};

    let relay_transaction = suite.create_relay_transaction(
        NON_USER_PRIV,
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

    let execute_msg_err: ContractError = suite
        .app
        .execute_contract(relayer, wallet_address, &relay_msg, &[])
        .unwrap_err()
        .downcast()
        .unwrap();

    assert_eq!(execute_msg_err, ContractError::IsNotUser {});
}
