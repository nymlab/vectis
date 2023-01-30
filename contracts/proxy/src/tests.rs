use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, BankMsg, BlockInfo, CosmosMsg, DepsMut, StdError, Timestamp};
use cw2::ContractVersion;

use crate::contract::{execute, instantiate, query_guardian_update_request, query_info};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::PENDING_GUARDIAN_ROTATION;

use vectis_wallet::{
    CreateWalletMsg, Guardians, GuardiansUpdateMsg, GuardiansUpdateRequest, WalletInfo,
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

const MULTISIG_CODE_ID: u64 = 13;

// this will set up the instantiation for other tests
// returns Controller address
fn do_instantiate(mut deps: DepsMut) -> Addr {
    let create_wallet_msg = CreateWalletMsg {
        controller_addr: "controller_addr".to_string(),
        guardians: get_guardians(),
        relayers: vec![RELAYER1.into(), RELAYER2.into()],
        proxy_initial_funds: vec![],
        label: "initial label".to_string(),
    };

    let instantiate_msg = InstantiateMsg {
        create_wallet_msg,
        multisig_code_id: MULTISIG_CODE_ID,
        code_id: 0,
    };

    let info = mock_info("creator", &[]);
    let env = mock_env();

    instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
    let info = query_info(deps.as_ref()).unwrap();

    let expected_info = WalletInfo {
        controller_addr: Addr::unchecked("controller_addr"),
        factory: Addr::unchecked("creator"),
        nonce: 0,
        version: ContractVersion {
            contract: "crates.io:smart-contract-wallet-proxy".to_string(),
            version: "0.1.0".to_string(),
        },
        code_id: 0,
        multisig_code_id: MULTISIG_CODE_ID,
        guardians: vec![Addr::unchecked(GUARD1), Addr::unchecked(GUARD2)],
        relayers: vec![Addr::unchecked(RELAYER2), Addr::unchecked(RELAYER1)],
        is_frozen: false,
        multisig_address: None,
        label: "initial label".to_string(),
    };

    assert_eq!(expected_info, info);
    Addr::unchecked("controller_addr")
}

#[test]
fn controller_cannot_be_a_guardian() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

    let guardians_with_controller = Guardians {
        addresses: vec![GUARD1.to_string(), "controller_addr".to_string()],
        guardians_multisig: None,
    };

    let create_wallet_msg = CreateWalletMsg {
        controller_addr: "controller_addr".to_string(),
        guardians: guardians_with_controller,
        relayers: vec![RELAYER1.into(), RELAYER2.into()],
        proxy_initial_funds: vec![],
        label: "initial label".to_string(),
    };

    let instantiate_msg = InstantiateMsg {
        create_wallet_msg,
        multisig_code_id: MULTISIG_CODE_ID,
        code_id: 0,
    };

    let info = mock_info("creator", &[]);
    let env = mock_env();

    let err = instantiate(deps.as_mut().branch(), env, info, instantiate_msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::generic_err("controller cannot be a guardian"))
    );
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
    assert_eq!(response.events[0].attributes, [("status", "frozen")]);

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(wallet_info.is_frozen);

    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(response.events[0].attributes, [("status", "unfrozen")]);

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
fn frozen_contract_cannot_execute_controller_tx() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(response.events[0].attributes, [("status", "frozen")]);

    let exec_msg: ExecuteMsg = ExecuteMsg::Execute {
        msgs: vec![CosmosMsg::Bank(BankMsg::Burn {
            amount: coins(1, "ucosm"),
        })],
    };
    let exec_err = execute(deps.as_mut(), env, info, exec_msg).unwrap_err();
    assert_eq!(exec_err, ContractError::Frozen {});
}

#[test]
fn frozen_contract_controller_cannot_rotate_guardians_or_controller() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    execute(deps.as_mut(), env, info, msg).unwrap();

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateGuardians {};

    let err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
    assert_eq!(err, ContractError::Frozen {});

    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: "new controller".into(),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Frozen {});
}

#[test]
fn frozen_contract_guardians_can_rotate_controller_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    execute(deps.as_mut(), env, info, msg).unwrap();
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(wallet_info.is_frozen);

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();

    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: "new controller123".into(),
    };

    execute(deps.as_mut(), env, info, msg).unwrap();
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert_ne!(wallet_info.controller_addr, controller_addr);
}

#[test]
fn frozen_contract_cannot_create_update_guardians_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(GUARD1, &[]);
    let env = mock_env();
    let msg = ExecuteMsg::RevertFreezeStatus {};
    execute(deps.as_mut(), env, info, msg).unwrap();

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let request = GuardiansUpdateMsg {
        guardians: Guardians {
            addresses: vec![GUARD1.to_string(), GUARD3.to_string()],
            guardians_multisig: None,
        },
        new_multisig_code_id: None,
    };

    let msg = ExecuteMsg::RequestUpdateGuardians {
        request: Some(request),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::Frozen {});
}

#[test]
fn controller_cannot_create_update_guardians_request_to_include_self() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(wallet_info.guardians.contains(&Addr::unchecked(GUARD2)));
    assert!(!wallet_info.guardians.contains(&Addr::unchecked(GUARD3)));

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let new_guardians = Guardians {
        addresses: vec![controller_addr.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };

    let request = GuardiansUpdateMsg {
        guardians: new_guardians,
        new_multisig_code_id: None,
    };

    let msg = ExecuteMsg::RequestUpdateGuardians {
        request: Some(request),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::Std(StdError::generic_err("controller cannot be a guardian"))
    );
}

#[test]
fn controller_cannot_execute_not_active_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let guardians = Guardians {
        addresses: vec![controller_addr.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };

    let request = GuardiansUpdateRequest::new(guardians, None, &env.block);
    PENDING_GUARDIAN_ROTATION
        .save(deps.as_mut().storage, &request)
        .unwrap();

    let msg = ExecuteMsg::UpdateGuardians {};

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::GuardianRequestNotExecutable {});
}

#[test]
fn controller_cannot_execute_update_guardian_when_no_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let msg = ExecuteMsg::UpdateGuardians {};

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::GuardianRequestNotFound {});
}

#[test]
fn controller_can_execute_active_guardian_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let guardians = Guardians {
        addresses: vec![controller_addr.to_string(), GUARD3.to_string()],
        guardians_multisig: None,
    };

    let mock_block = BlockInfo {
        height: 12_345,
        time: Timestamp::from_nanos(571_797_419_879_305_533),
        chain_id: "cosmos-testnet-14002".to_string(),
    };

    let request = GuardiansUpdateRequest::new(guardians, None, &mock_block);
    PENDING_GUARDIAN_ROTATION
        .save(deps.as_mut().storage, &request)
        .unwrap();

    let msg = ExecuteMsg::UpdateGuardians {};

    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [
            ("guardians", format!("{:?}", request.guardians.addresses)),
            ("multisig", "false".to_string())
        ]
    );
}

#[test]
fn controller_can_create_update_guardians_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let request = GuardiansUpdateMsg {
        guardians: Guardians {
            addresses: vec![GUARD1.to_string(), GUARD3.to_string()],
            guardians_multisig: None,
        },
        new_multisig_code_id: None,
    };

    let msg = ExecuteMsg::RequestUpdateGuardians {
        request: Some(request.clone()),
    };

    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [
            ("create", "true".to_string()),
            ("guardians", format!("{:?}", request.guardians.addresses))
        ]
    );

    let query_request = query_guardian_update_request(deps.as_ref())
        .unwrap()
        .unwrap();

    assert_eq!(query_request.guardians, request.guardians)
}

#[test]
fn controller_can_remove_update_guardians_request() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let msg = ExecuteMsg::RequestUpdateGuardians { request: None };

    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(response.events[0].attributes, [("create", "false")]);

    let query_request = query_guardian_update_request(deps.as_ref()).unwrap();

    assert!(query_request.is_none())
}

#[test]
fn controller_can_add_relayer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let mut wallet_info = query_info(deps.as_ref()).unwrap();

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let new_relayer_address = Addr::unchecked(RELAYER3);
    let msg = ExecuteMsg::AddRelayer {
        new_relayer_address: new_relayer_address.clone(),
    };

    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [("address", new_relayer_address.clone().as_str())]
    );

    wallet_info.relayers.push(new_relayer_address);
    let new_wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!(wallet_info.relayers, new_wallet_info.relayers);
}

#[test]
fn controller_can_remove_relayer() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially we have a wallet with 2 relayers
    let mut wallet_info = query_info(deps.as_ref()).unwrap();

    let info = mock_info(controller_addr.as_str(), &[]);
    let env = mock_env();

    let relayer_address = Addr::unchecked(RELAYER2);
    let msg = ExecuteMsg::RemoveRelayer {
        relayer_address: relayer_address.clone(),
    };

    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [("address", relayer_address.clone().as_str())]
    );

    wallet_info
        .relayers
        .retain(|relayer| *relayer != relayer_address);
    let new_wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!(wallet_info.relayers, new_wallet_info.relayers);
}

#[test]
fn guardian_can_rotate_controller_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    // GUARD1 is a valid guardian
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: new_address.to_string(),
    };
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [
            ("old_address", wallet_info.controller_addr.as_str()),
            ("new_address", new_address)
        ]
    );

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(new_address.eq(wallet_info.controller_addr.as_str()));
}

#[test]
fn controller_can_update_label() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    let info = mock_info(controller_addr.as_ref(), &[]);
    let env = mock_env();

    let new_label = "new label";
    let msg = ExecuteMsg::UpdateLabel {
        new_label: new_label.to_string(),
    };

    let response = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(response.events[0].attributes, [("label", new_label)]);

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!(new_label, wallet_info.label.as_str());
}

#[test]
fn non_controller_update_label_fails() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let _ = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    let info = mock_info("Non-controller", &[]);
    let env = mock_env();

    let new_label = "new label";
    let msg = ExecuteMsg::UpdateLabel {
        new_label: new_label.to_string(),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::IsNotController {});

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert_eq!("initial label", wallet_info.label.as_str());
}

#[test]
fn controller_can_rotate_controller_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    let info = mock_info(controller_addr.as_ref(), &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: new_address.to_string(),
    };
    let response = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        response.events[0].attributes,
        [
            ("old_address", controller_addr.to_string()),
            ("new_address", new_address.to_string())
        ]
    );

    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(new_address.eq(wallet_info.controller_addr.as_str()));
}

#[test]
fn invalid_guardian_or_use_cannot_rotate_controller_key() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    do_instantiate(deps.as_mut());

    // INVALID_GUARD is not a valid guardian
    let info = mock_info(INVALID_GUARD, &[]);
    let env = mock_env();

    let new_address = "new_key";
    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: new_address.to_string(),
    };

    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_err());
}

#[test]
fn rotate_controller_key_same_address_fails() {
    let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
    let controller_addr = do_instantiate(deps.as_mut());

    // initially it is not frozen
    let wallet_info = query_info(deps.as_ref()).unwrap();
    assert!(!wallet_info.is_frozen);

    // GUARD1 is a valid guardian
    let info = mock_info(GUARD1, &[]);
    let env = mock_env();

    // Make an attempt to provide the same address as currently set
    let msg = ExecuteMsg::RotateControllerKey {
        new_controller_address: controller_addr.to_string(),
    };

    let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    assert_eq!(err, ContractError::AddressesAreEqual {});
}
