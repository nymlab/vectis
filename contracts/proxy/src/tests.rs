use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coins, to_binary, Addr, BankMsg, Binary, CosmosMsg, DepsMut};

use crate::contract::{execute, execute_relay, instantiate, query_info};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

use secp256k1::bitcoin_hashes::sha256;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use vectis_wallet::{
    pub_key_to_address, CreateWalletMsg, Guardians, RelayTransaction, RelayTxError,
};

const GUARD1: &str = "guardian1";
const GUARD2: &str = "guardian2";
const GUARD3: &str = "guardian3";

fn get_guardians() -> Guardians {
    Guardians {
        addresses: vec![GUARD1.to_string(), GUARD2.to_string()],
        guardians_multisig: None,
    }
}

const RELAYER1: &str = "relayer1";
const RELAYER2: &str = "relayer2";
const RELAYER3: &str = "relayer3";

const INVALID_GUARD: &str = "not_a_guardian";
const INVALID_RELAYER: &str = "not_a_relayer";

const USER_PRIV: &[u8; 32] = &[
    239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];

const INVAILID_USER_PRIV: &[u8; 32] = &[
    239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 27,
];

const MULTISIG_CODE_ID: u64 = 13;

// this will set up the instantiation for other tests
// returns User address
fn do_instantiate(mut deps: DepsMut) -> Addr {
    let secp = Secp256k1::new();

    let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_serialized = Binary(public_key.serialize_uncompressed().to_vec());

    let create_wallet_msg = CreateWalletMsg {
        user_pubkey: public_key_serialized.clone(),
        guardians: get_guardians(),
        relayers: vec![RELAYER1.into(), RELAYER2.into()],
        proxy_initial_funds: vec![],
    };

    let instantiate_msg = InstantiateMsg {
        create_wallet_msg,
        multisig_code_id: MULTISIG_CODE_ID,
        code_id: 0,
        addr_prefix: "wasm".to_string(),
    };

    let info = mock_info("creator", &[]);
    let env = mock_env();

    let address = pub_key_to_address(&deps, "wasm", &public_key_serialized).unwrap();
    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    address
}

#[test]
fn guardian_can_revert_freeze_status() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    // GUARD1 is a valid guardian
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.attributes, [("action", "frozen")]);

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(wallet_info.is_frozen);

    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(response.attributes, [("action", "unfrozen")]);

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);
}

#[test]
fn non_guardian_cannot_revert_freeze_status() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // INVALID_GUARD is not a valid guardian
    let info = mock_info(INVALID_GUARD, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::IsNotGuardian {});
}

#[test]
fn frozen_contract_cannot_execute_user_tx() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.attributes, [("action", "frozen")]);

    let exec_msg: ExecuteMsg = ExecuteMsg::Execute {
        msgs: vec![CosmosMsg::Bank(BankMsg::Burn {
            amount: coins(1, "ucosm"),
        })],
    };
    let exec_err = execute(deps.as_mut(), env, info, exec_msg).unwrap_err();
    assert_eq!(exec_err, ContractError::Frozen {});
}

#[test]
fn frozen_contract_user_cannot_rotate_guardians() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.attributes, [("action", "frozen")]);

    let info = mock_info(user_addr.as_str(), &[]);
    let env = mock_env();

    let new_guardians = Guardians {
        addresses: vec![GUARD1.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };
    let msg = ExecuteMsg::UpdateGuardians {
        guardians: new_guardians,
        new_multisig_code_id: None,
    };

    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::Frozen {});
}

#[test]
fn user_can_update_non_multisig_guardian() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(wallet_info.guardians.contains(&Addr::unchecked(GUARD2)));
    assert!(!wallet_info.guardians.contains(&Addr::unchecked(GUARD3)));

    let info = mock_info(user_addr.as_str(), &[]);
    let env = mock_env();

    let new_guardians = Guardians {
        addresses: vec![GUARD1.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };
    let msg = ExecuteMsg::UpdateGuardians {
        guardians: new_guardians,
        new_multisig_code_id: None,
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("action", "Updated wallet guardians: Non-Multisig")]
    );

    // Ensure relayer is added successfully
    let new_wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!new_wallet_info.guardians.contains(&Addr::unchecked(GUARD2)));
    assert!(new_wallet_info.guardians.contains(&Addr::unchecked(GUARD3)));
}

#[test]
fn user_can_add_relayer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let mut wallet_info = query_info(deps.as_ref()).unwrap();

    let info = mock_info(user_addr.as_str(), &[]);
    let env = mock_env();

    let new_relayer_address = Addr::unchecked(RELAYER3);
    let msg = ExecuteMsg::AddRelayer {
        new_relayer_address: new_relayer_address.clone(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("action", format!("Relayer {:?} added", new_relayer_address))]
    );

    // Ensure relayer is added successfully
    wallet_info.relayers.push(new_relayer_address);
    let new_wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!(wallet_info.relayers, new_wallet_info.relayers);
}

#[test]
fn user_can_remove_relayer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let mut wallet_info = query_info(deps.as_ref()).unwrap();

    let info = mock_info(user_addr.as_str(), &[]);
    let env = mock_env();

    let relayer_address = Addr::unchecked(RELAYER2);
    let msg = ExecuteMsg::RemoveRelayer {
        relayer_address: relayer_address.clone(),
    };

    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(
        response.attributes,
        [("action", format!("Relayer {:?} removed", relayer_address))]
    );

    // Ensure relayer is removed successfully
    wallet_info.relayers = wallet_info
        .relayers
        .into_iter()
        .filter(|relayer| *relayer != relayer_address)
        .collect();
    let new_wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!(wallet_info.relayers, new_wallet_info.relayers);
}

#[test]
fn guardian_can_rotate_user_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    // GUARD1 is a valid guardian
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateUserKey {
        new_user_address: new_address.to_string(),
    };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.attributes, [("action", "execute_rotate_user_key")]);

    // Ensure key is rotated successfully
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(new_address.eq(wallet_info.user_addr.as_str()));
}

#[test]
fn user_can_rotate_user_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    let info = mock_info(&user_addr.to_string(), &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateUserKey {
        new_user_address: new_address.to_string(),
    };
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.attributes, [("action", "execute_rotate_user_key")]);

    // Ensure key is rotated successfully
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(new_address.eq(wallet_info.user_addr.as_str()));
}

#[test]
fn invalid_guardian_or_use_cannot_rotate_user_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // INVALID_GUARD is not a valid guardian
    let info = mock_info(INVALID_GUARD, &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateUserKey {
        new_user_address: new_address.to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_err());
}

#[test]
fn rotate_user_key_same_address_fails() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let user_addr = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    // GUARD1 is a valid guardian
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();

    // Make an attempt to provide the same address as currently set
    let msg = ExecuteMsg::RotateUserKey {
        new_user_address: user_addr.to_string(),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::AddressesAreEqual {});
}

#[test]
fn relay_proxy_user_tx_succeeds() {
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
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
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
