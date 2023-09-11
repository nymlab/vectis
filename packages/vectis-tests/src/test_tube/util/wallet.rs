use crate::{
    constants::*,
    test_tube::util::{
        contract::Contract,
        passkey::{
            de_client_data, hash_to_base64url_string, hash_to_hex_string, must_create_credential,
            must_get_credential,
        },
    },
};
use cosmwasm_std::{coin, to_binary, Addr, BankMsg, Binary, CosmosMsg, Empty};
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContractResponse;
use osmosis_test_tube::OsmosisTestApp;
use test_tube::{RunnerExecuteResult, SigningAccount};
use vectis_wallet::{
    interface::{
        factory_service_trait::{
            ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
        },
        wallet_plugin_trait::ExecMsg as WalletPluginExecMsg,
        wallet_trait::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    },
    types::{
        authenticator::{
            Authenticator, AuthenticatorProvider, AuthenticatorType, EmptyInstantiateMsg,
        },
        entity::Entity,
        factory::CreateWalletMsg,
        plugin::{PluginInstallParams, PluginPermission, PluginSource},
        wallet::{Nonce, RelayTransaction, VectisRelayedTx, WalletInfo, WebauthnRelayedTxMsg},
    },
};

pub fn default_entity() -> Entity {
    Entity {
        auth: Authenticator {
            ty: AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: to_binary(&"data").unwrap(),
        nonce: 0,
    }
}

pub fn webauthn_entity(data: &[u8]) -> Entity {
    Entity {
        auth: Authenticator {
            ty: AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: Binary::from(data),
        nonce: 0,
    }
}

pub fn create_webauthn_wallet<'a>(
    app: &'a OsmosisTestApp,
    factory_addr: &'a str,
    vid: &'a str,
    init_balance: u128,
    signer: &'a SigningAccount,
) -> (Addr, Vec<u8>) {
    let pubkey = must_create_credential(vid);
    let entity = webauthn_entity(&pubkey);

    let create_msg = FactoryServiceExecMsg::CreateWallet {
        create_wallet_msg: CreateWalletMsg {
            controller: entity.clone(),
            relayers: vec![],
            proxy_initial_funds: vec![coin(init_balance, DENOM)],
            vid: vid.into(),
            initial_data: vec![],
            plugins: vec![],
        },
    };

    let factory = Contract::from_addr(&app, factory_addr.into());
    factory
        .execute(
            &create_msg,
            &[coin(WALLET_FEE + INIT_BALANCE, DENOM)],
            signer,
        )
        .unwrap();

    let wallet_addr: Option<Addr> = factory
        .query(&FactoryServiceQueryMsg::WalletByVid { vid: vid.into() })
        .unwrap();

    (wallet_addr.unwrap(), pubkey)
}

pub fn sign_and_submit(
    app: &OsmosisTestApp,
    messages: Vec<CosmosMsg>,
    vid: &str,
    wallet_addr: &str,
    relayer: &SigningAccount,
) -> RunnerExecuteResult<MsgExecuteContractResponse> {
    let wallet = Contract::from_addr(&app, wallet_addr.into());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let relay_tx = sign_and_create_relay_tx(messages, info.controller.nonce, vid);

    wallet.execute(
        &WalletExecMsg::AuthExec {
            transaction: relay_tx,
        },
        &[],
        relayer,
    )
}

pub fn sign_and_create_relay_tx(
    messages: Vec<CosmosMsg>,
    nonce: Nonce,
    vid: &str,
) -> RelayTransaction {
    let signed_msg = VectisRelayedTx {
        messages,
        nonce,
        sponsor_fee: None,
    };
    let signed_msg_str = serde_json_wasm::to_string(&signed_msg).unwrap();

    // We encode the hash vec into hex to pass to the cli
    let challenge = hash_to_hex_string(signed_msg_str.as_bytes());
    let response = must_get_credential(vid, challenge.clone());

    // Compare the expeted challenge value to the one returned from passkey
    let expect_challenge = hash_to_base64url_string(signed_msg_str.as_bytes());
    let challenge_in_client_data = de_client_data(&response.client_data_json);
    assert_eq!(expect_challenge, challenge_in_client_data.challenge);

    RelayTransaction {
        message: to_binary(&WebauthnRelayedTxMsg {
            signed_data: signed_msg_str,
            auth_data: Binary::from(response.authenticator_data),
            client_data: Binary::from(response.client_data_json),
        })
        .unwrap(),
        signature: Binary::from(response.signature),
    }
}

pub fn add_test_plugin(
    app: &OsmosisTestApp,
    vid: &str,
    wallet_addr: &str,
    relayer: &SigningAccount,
    plugin_id: u64,
) -> RunnerExecuteResult<MsgExecuteContractResponse> {
    let wallet = Contract::from_addr(&app, wallet_addr.to_string());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let permission = match plugin_id {
        1 => PluginPermission::PreTxCheck,
        2 => PluginPermission::PostTxHook,
        3 => PluginPermission::Exec,
        // allow random number through for testing
        _ => PluginPermission::PreTxCheck,
    };

    let install_plugin_msg = WalletPluginExecMsg::InstallPlugins {
        install: vec![PluginInstallParams {
            src: PluginSource::VectisRegistry(plugin_id, None),
            permission,
            label: "plugin_install".into(),
            funds: vec![],
            instantiate_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
        }],
    };
    let relay_tx = sign_and_create_relay_tx(
        vec![CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: wallet_addr.to_string(),
            msg: to_binary(&install_plugin_msg).unwrap(),
            funds: vec![],
        })],
        info.controller.nonce,
        vid,
    );

    wallet.execute(
        &WalletExecMsg::AuthExec {
            transaction: relay_tx,
        },
        &[],
        relayer,
    )
}
