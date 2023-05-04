pub use anyhow::{anyhow, Result};
pub use cosmwasm_std::testing::mock_dependencies;
pub use cosmwasm_std::{
    coin, to_binary, Addr, Binary, BlockInfo, CanonicalAddr, Coin, CosmosMsg, Empty, Event,
    QueryRequest, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
pub use cw20::BalanceResponse;
pub use cw3::VoterListResponse;
pub use cw3_fixed_multisig::contract::{
    execute as fixed_multisig_execute, instantiate as fixed_multisig_instantiate,
    query as fixed_multisig_query,
};
pub use cw3_fixed_multisig::msg::QueryMsg as MultiSigQueryMsg;
pub use cw3_flex_multisig::{
    contract::{
        execute as flex_multisig_execute, instantiate as flex_multisig_instantiate,
        query as flex_multisig_query,
    },
    msg::{ExecuteMsg as cw3flexExecMsg, InstantiateMsg as cw3flexInstMsg},
};
pub use cw4::Member;
pub use cw4_group::{
    contract::{execute as cw4_execute, instantiate as cw4_instantiate, query as cw4_query},
    msg::{ExecuteMsg as cw4ExecMsg, InstantiateMsg as cw4InstMsg},
};
pub use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
pub use cw_utils::{Duration, Expiration};
pub use derivative::Derivative;
pub use secp256k1::{bitcoin_hashes::sha256, Message, PublicKey, Secp256k1, SecretKey};
pub use serde::{de::DeserializeOwned, Serialize};

pub use vectis_plugin_registry::{
    contract::{
        ExecMsg as PRegistryExecMsg, InstantiateMsg as PRegistryInstantiateMsg, Plugin,
        PluginRegistry, QueryMsg as PRegistryQueryMsg,
    },
    error::ContractError as PRegistryContractError,
    responses::{ConfigResponse, PluginsResponse},
};

pub use vectis_factory::contract::{
    execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
    reply as factory_reply,
};
pub use vectis_proxy::contract::{
    execute as proxy_execute, instantiate as proxy_instantiate, migrate as proxy_migrate,
    query as proxy_query, reply as proxy_reply,
};

pub use vectis_wallet::{
    pub_key_to_address, CodeIdType, CreateWalletMsg, Guardians, GuardiansUpdateMsg,
    GuardiansUpdateRequest, MultiSig, PluginListResponse, PluginParams, ProxyExecuteMsg,
    ProxyQueryMsg, RelayTransaction, ThresholdAbsoluteCount, VectisActors, WalletFactoryExecuteMsg,
    WalletFactoryExecuteMsg as FactoryExecuteMsg, WalletFactoryInstantiateMsg as InstantiateMsg,
    WalletFactoryQueryMsg as FactoryQueryMsg,
};

/// This is used for staking queries
/// https://github.com/CosmWasm/cosmwasm/blob/32f308a1a56ae5b8278947891306f7a374c3df94/packages/vm/src/environment.rs#L383
pub const DENOM: &str = "TOKEN";
pub const MINT_AMOUNT: u128 = 2u128;
pub const WALLET_FEE: u128 = 0u128;
pub const REGISTRY_FEE: u128 = 0u128;
pub const INSTALL_FEE: u128 = 0u128;
pub const MINTER_CAP: u128 = 10000;
pub const CONTROLLER_PRIV: &[u8; 32] = &[
    239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];
pub const NON_CONTROLLER_PRIV: &[u8; 32] = &[
    239, 111, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];
pub const CONTROLLER_ADDR: &str = "wasm1ky4epcqzk0mngu7twqz06qzmpgrxstxhfch6yl";
pub const MULTISIG_THRESHOLD: ThresholdAbsoluteCount = 2;
pub const PROP_APPROVER: &str = "approver";
pub const GUARD1: &str = "guardian1";
pub const GUARD2: &str = "guardian2";
pub const GUARD3: &str = "guardian3";
pub fn contract_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(factory_execute, factory_instantiate, factory_query)
        .with_reply(factory_reply);
    Box::new(contract)
}

pub fn contract_proxy() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(proxy_execute, proxy_instantiate, proxy_query)
        .with_migrate(proxy_migrate)
        .with_reply(proxy_reply);
    Box::new(contract)
}

pub fn contract_fixed_multisig() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        fixed_multisig_execute,
        fixed_multisig_instantiate,
        fixed_multisig_query,
    );
    Box::new(contract)
}

pub fn contract_flex_multisig() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        flex_multisig_execute,
        flex_multisig_instantiate,
        flex_multisig_query,
    );
    Box::new(contract)
}

pub fn contract_cw4() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(cw4_execute, cw4_instantiate, cw4_query);
    Box::new(contract)
}
pub fn contract_plugin_registry() -> Box<dyn Contract<Empty>> {
    Box::new(PluginRegistry::new())
}

pub fn proxy_exec(to_contract: &Addr, msg: &impl Serialize, funds: Vec<Coin>) -> ProxyExecuteMsg {
    ProxyExecuteMsg::Execute {
        msgs: vec![CosmosMsg::<Empty>::Wasm(WasmMsg::Execute {
            contract_addr: to_contract.to_string(),
            msg: to_binary(&msg).unwrap(),
            funds,
        })],
    }
}
