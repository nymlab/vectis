use cosmwasm_std::{coin, to_binary, Coin, CosmosMsg, Event, Uint128, WasmMsg};
use osmosis_test_tube::{
    Account, Bank, Module, OsmosisTestApp, RunnerError, RunnerExecuteResult, RunnerResult,
    SigningAccount, Wasm,
};

use super::contract::Contract;
use vectis_wallet::types::{
    authenticator::{
        Authenticator, AuthenticatorProvider, AuthenticatorType, WebauthnInstantaiteMsg,
    },
    entity::Entity,
    factory::{AuthenticatorInstInfo, CreateWalletMsg, WalletFactoryInstantiateMsg},
    wallet::Controller,
};

pub struct Wallet<'a> {
    controller: Controller,
    addr: String,
    contract: Contract<'a>,
}

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

pub fn create_wallet_msg(vid: String) -> CreateWalletMsg {
    let controller = default_entity();
    CreateWalletMsg {
        controller,
        relayers: vec![],
        proxy_initial_funds: vec![],
        vid,
        initial_data: vec![],
        plugins: vec![],
    }
}
