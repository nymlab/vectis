use crate::{
    constants::*,
    helpers::{sign_and_create_relay_tx, webauthn_entity},
    passkey::*,
    test_tube::util::contract::Contract,
};
use cosmwasm_std::{coin, to_binary, Addr, CosmosMsg};
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContractResponse;
use osmosis_test_tube::OsmosisTestApp;
use test_tube::{RunnerExecuteResult, SigningAccount};
use vectis_wallet::{
    interface::{
        factory_service_trait::sv::{
            ExecMsg as FactoryServiceExecMsg, QueryMsg as FactoryServiceQueryMsg,
        },
        wallet_plugin_trait::sv::ExecMsg as WalletPluginExecMsg,
        wallet_trait::sv::{ExecMsg as WalletExecMsg, QueryMsg as WalletQueryMsg},
    },
    types::{
        authenticator::EmptyInstantiateMsg,
        factory::{CreateWalletMsg, MigrateWalletMsg},
        plugin::{PluginInstallParams, PluginPermission, PluginSource},
        wallet::WalletInfo,
    },
};

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
            chains: None,
            code_id: None,
        },
    };

    let factory = Contract::from_addr(app, factory_addr.into());
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
    let wallet = Contract::from_addr(app, wallet_addr.into());
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

pub fn sign_and_submit_auth_tx_without_plugins(
    app: &OsmosisTestApp,
    messages: Vec<CosmosMsg>,
    vid: &str,
    wallet_addr: &str,
    relayer: &SigningAccount,
) -> RunnerExecuteResult<MsgExecuteContractResponse> {
    let wallet = Contract::from_addr(app, wallet_addr.into());
    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let relay_tx = sign_and_create_relay_tx(messages, info.controller.nonce, vid);

    wallet.execute(
        &WalletExecMsg::AuthExecWithoutPlugins {
            transaction: relay_tx,
        },
        &[],
        relayer,
    )
}

pub fn sign_migration_msg(
    app: &OsmosisTestApp,
    messages: Vec<CosmosMsg>,
    vid: &str,
    wallet_addr: &str,
    factory_addr: &str,
    relayer: &SigningAccount,
) -> RunnerExecuteResult<MsgExecuteContractResponse> {
    let wallet = Contract::from_addr(app, wallet_addr.into());
    let factory = Contract::from_addr(app, factory_addr.into());

    let info: WalletInfo = wallet.query(&WalletQueryMsg::Info {}).unwrap();

    let relay_tx = sign_and_create_relay_tx(messages, info.controller.nonce, vid);

    factory.execute(
        &FactoryServiceExecMsg::MigrateWallet {
            migrations_msg: MigrateWalletMsg {
                addr_to_migrate: wallet_addr.into(),
                tx: relay_tx,
            },
        },
        &[],
        relayer,
    )
}

pub fn add_test_plugin(
    app: &OsmosisTestApp,
    vid: &str,
    wallet_addr: &str,
    relayer: &SigningAccount,
    plugin_id: u64,
) -> RunnerExecuteResult<MsgExecuteContractResponse> {
    let wallet = Contract::from_addr(app, wallet_addr.to_string());
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
