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
pub use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
pub use cw_utils::{Duration, Expiration};
pub use derivative::Derivative;
pub use secp256k1::{bitcoin_hashes::sha256, Message, PublicKey, Secp256k1, SecretKey};
pub use serde::{de::DeserializeOwned, Serialize};

pub use cw_core::{
    contract::{
        execute as dao_execute, instantiate as dao_instantiate, query as dao_query,
        reply as dao_reply,
    },
    msg::{Admin, InstantiateMsg as DaoInstMsg, ModuleInstantiateInfo, QueryMsg as DaoQueryMsg},
};

pub use cw_proposal_single::{
    contract::{
        execute as prop_execute, instantiate as prop_instantiate, query as prop_query,
        reply as prop_reply,
    },
    msg::{DepositInfo, DepositToken, InstantiateMsg as PropInstMsg, QueryMsg as PropQueryMsg},
    query::{ProposalListResponse, ProposalResponse},
};

pub use cw20_staked_balance_voting::{
    contract::{
        execute as vote_execute, instantiate as vote_instantiate, query as vote_query,
        reply as vote_reply,
    },
    msg::{
        ActiveThreshold, InstantiateMsg as VoteInstMsg, QueryMsg as VoteQueryMsg, StakingInfo,
        TokenInfo,
    },
};

pub use cw20_stake::{
    contract::{execute as stake_execute, instantiate as stake_instantiate, query as stake_query},
    msg::{
        InstantiateMsg as StakeInstMsg, QueryMsg as StakeQueryMsg, ReceiveMsg,
        StakedBalanceAtHeightResponse,
    },
};

pub use vectis_dao_tunnel::{
    contract::{
        execute as dtunnel_execute, instantiate as dtunnel_instantiate, query as dtunnel_query,
        reply as dtunnel_reply,
    },
    msg::ExecuteMsg as DTunnelExecuteMsg,
    msg::InstantiateMsg as DTunnelInstanstiateMsg,
};

pub use vectis_plugin_registry::{
    contract::{
        ExecMsg as PRegistryExecMsg, InstantiateMsg as PRegistryInstantiateMsg, Plugin,
        PluginRegistry, QueryMsg as PRegistryQueryMsg,
    },
    error::ContractError as PRegistryContractError,
    responses::{ConfigResponse, PluginsResponse},
};
pub use vectis_remote_tunnel::{
    contract::{
        execute as rtunnel_execute, instantiate as rtunnel_instantiate, query as rtunnel_query,
        reply as rtunnel_reply,
    },
    msg::ExecuteMsg as RTunnelExecuteMsg,
    msg::InstantiateMsg as RTunnelInstanstiateMsg,
};

pub use vectis_factory::contract::{
    execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
    reply as factory_reply,
};

pub use vectis_remote_factory::contract::{
    execute as remote_factory_execute, instantiate as remote_factory_instantiate,
    query as remote_factory_query, reply as remote_factory_reply,
};

pub use vectis_govec::{
    contract::{execute as govec_execute, instantiate as govec_instantiate, query as govec_query},
    msg::InstantiateMsg as GovecInstantiateMsg,
};

pub use vectis_proxy::contract::{
    execute as proxy_execute, instantiate as proxy_instantiate, migrate as proxy_migrate,
    query as proxy_query, reply as proxy_reply,
};

pub use vectis_wallet::{
    pub_key_to_address, CodeIdType, CreateWalletMsg, GovecExecuteMsg, GovecQueryMsg, Guardians,
    GuardiansUpdateMsg, GuardiansUpdateRequest, MultiSig, PluginParams, ProposalExecuteMsg,
    ProxyExecuteMsg, ProxyQueryMsg, RelayTransaction, StakeExecuteMsg, ThresholdAbsoluteCount,
    UnclaimedWalletList, WalletFactoryExecuteMsg, WalletFactoryExecuteMsg as FactoryExecuteMsg,
    WalletFactoryInstantiateMsg as InstantiateMsg, WalletFactoryQueryMsg as FactoryQueryMsg,
    GOVEC_CLAIM_DURATION_DAY_MUL,
};

pub const MINT_AMOUNT: u128 = 2u128;
pub const WALLET_FEE: u128 = 10u128;
pub const REGISTRY_FEE: u128 = 100u128;
pub const CLAIM_FEE: u128 = 10u128;
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
pub const GUARD1: &str = "guardian1";
pub const GUARD2: &str = "guardian2";
pub const GUARD3: &str = "guardian3";
pub fn contract_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(factory_execute, factory_instantiate, factory_query)
        .with_reply(factory_reply);
    Box::new(contract)
}

pub fn contract_remote_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        remote_factory_execute,
        remote_factory_instantiate,
        remote_factory_query,
    )
    .with_reply(remote_factory_reply);
    Box::new(contract)
}

pub fn contract_proxy() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(proxy_execute, proxy_instantiate, proxy_query)
        .with_migrate(proxy_migrate)
        .with_reply(proxy_reply);
    Box::new(contract)
}

pub fn contract_govec() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(govec_execute, govec_instantiate, govec_query);
    Box::new(contract)
}

pub fn contract_stake() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(stake_execute, stake_instantiate, stake_query);
    Box::new(contract)
}

pub fn contract_vote() -> Box<dyn Contract<Empty>> {
    let contract =
        ContractWrapper::new(vote_execute, vote_instantiate, vote_query).with_reply(vote_reply);
    Box::new(contract)
}

pub fn contract_dao() -> Box<dyn Contract<Empty>> {
    let contract =
        ContractWrapper::new(dao_execute, dao_instantiate, dao_query).with_reply(dao_reply);
    Box::new(contract)
}

pub fn contract_proposal() -> Box<dyn Contract<Empty>> {
    let contract =
        ContractWrapper::new(prop_execute, prop_instantiate, prop_query).with_reply(prop_reply);
    Box::new(contract)
}

pub fn contract_multisig() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        fixed_multisig_execute,
        fixed_multisig_instantiate,
        fixed_multisig_query,
    );
    Box::new(contract)
}

pub fn contract_dao_tunnel() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(dtunnel_execute, dtunnel_instantiate, dtunnel_query)
        .with_reply(dtunnel_reply);
    Box::new(contract)
}

pub fn contract_remote_tunnel() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(rtunnel_execute, rtunnel_instantiate, rtunnel_query)
        .with_reply(rtunnel_reply);
    Box::new(contract)
}

pub fn contract_plugin_registry<'a>() -> Box<dyn Contract<Empty>> {
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
