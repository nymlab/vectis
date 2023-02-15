use cosmwasm_schema::cw_serde;
use cw2::set_contract_version;
use cw_storage_plus::{Bound, Item, Map};
use sylvia::{contract, schemars};

use cosmwasm_std::{
    ensure_eq, BankMsg, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo,
    Order, Response, StdResult,
};

use crate::{
    error::ContractError,
    responses::{ConfigResponse, PluginsResponse},
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

#[cw_serde]
pub struct Plugin {
    pub id: u64,
    pub name: String,
    pub creator: CanonicalAddr,
    pub ipfs_hash: String,
    pub version: String,
    pub code_id: u64,
    pub checksum: String,
}

pub struct PluginRegistry<'a> {
    pub(crate) total_plugins: Item<'a, u64>,
    pub(crate) registry_fee: Item<'a, Coin>,
    pub(crate) dao_addr: Item<'a, CanonicalAddr>,
    pub(crate) reviewer: Item<'a, CanonicalAddr>,
    pub(crate) plugins: Map<'a, u64, Plugin>,
}

#[contract]
impl PluginRegistry<'_> {
    pub const fn new() -> Self {
        Self {
            total_plugins: Item::new("total_plugins"),
            registry_fee: Item::new("registry_fee"),
            dao_addr: Item::new("dao_addr"),
            reviewer: Item::new("reviewer"),
            plugins: Map::new("plugins"),
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        registry_fee: Coin,
        dao_addr: String,
        reviewer: String,
    ) -> Result<Response, ContractError> {
        let (deps, ..) = ctx;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        let dao_addr = deps.api.addr_canonicalize(&dao_addr)?;

        let reviewer_multisig = deps
            .api
            .addr_canonicalize(deps.api.addr_validate(&reviewer)?.as_str())?;

        self.total_plugins.save(deps.storage, &0u64)?;
        self.registry_fee.save(deps.storage, &registry_fee)?;
        self.dao_addr.save(deps.storage, &dao_addr)?;
        self.reviewer.save(deps.storage, &reviewer_multisig)?;

        Ok(Response::default())
    }

    #[msg(exec)]
    #[allow(clippy::too_many_arguments)]
    pub fn register_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        name: String,
        creator: String,
        ipfs_hash: String,
        version: String,
        code_id: u64,
        checksum: String,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;

        // Check if the caller has enough funds to pay the fee
        let registry_fee = self.registry_fee.load(deps.storage)?;

        let fund = info
            .funds
            .iter()
            .find(|c| c.denom == registry_fee.denom)
            .ok_or(ContractError::RegistryFeeRequired)?;

        if fund.amount < registry_fee.amount {
            return Err(ContractError::InsufficientFee(
                registry_fee.amount,
                fund.amount,
            ));
        };

        // Check if the caller is a reviewer
        let reviewer = self.reviewer.load(deps.storage)?;
        if deps.api.addr_humanize(&reviewer)? != info.sender {
            return Err(ContractError::Unauthorized);
        }

        // Store plugin information in PLUGINS Map<u64(id), Plugin>
        let id = self
            .total_plugins
            .update(deps.storage, |total| -> StdResult<_> {
                Ok(total.checked_add(1).expect("overflow"))
            })?;

        let plugin = Plugin {
            id,
            name,
            creator: deps.api.addr_canonicalize(&creator)?,
            ipfs_hash,
            version,
            code_id,
            checksum,
        };

        self.plugins.save(deps.storage, id, &plugin)?;

        // Send funds to the DAO
        let msg = CosmosMsg::Bank(BankMsg::Send {
            to_address: deps
                .api
                .addr_humanize(&self.dao_addr.load(deps.storage)?)?
                .to_string(),
            amount: vec![self.registry_fee.load(deps.storage)?],
        });

        Ok(Response::new()
            .add_event(Event::new("vectis.plugin_registry.v1.MsgInstantiate"))
            .add_message(msg))
    }

    #[msg(exec)]
    pub fn unregister_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        id: u64,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;

        // Check if the caller is a reviewer
        let reviewer = self.reviewer.load(deps.storage)?;
        if deps.api.addr_humanize(&reviewer)? != info.sender {
            return Err(ContractError::Unauthorized);
        }

        // Remove plugin information from registry
        self.plugins.remove(deps.storage, id);
        self.total_plugins
            .update(deps.storage, |total| -> StdResult<_> {
                Ok(total.checked_sub(1).expect("underflow"))
            })?;

        let event = Event::new("vectis.plugin_registry.v1.MsgUnregisterPlugin")
            .add_attribute("plugin_id", id.to_string());

        Ok(Response::new().add_event(event))
    }

    #[msg(exec)]
    #[allow(clippy::too_many_arguments)]
    pub fn update_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        id: u64,
        name: Option<String>,
        creator: Option<String>,
        version: Option<String>,
        ipfs_hash: Option<String>,
        code_id: Option<u64>,
        checksum: Option<String>,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;

        let reviewer = self.reviewer.load(deps.storage)?;
        if deps.api.addr_humanize(&reviewer)? != info.sender {
            return Err(ContractError::Unauthorized);
        }

        let plugin = self
            .plugins
            .update(deps.storage, id, |plugin| -> StdResult<Plugin> {
                let p = plugin.unwrap();
                Ok(Plugin {
                    id,
                    name: name.unwrap_or(p.name),
                    creator: if creator.is_some() {
                        deps.api.addr_canonicalize(&creator.unwrap())?
                    } else {
                        p.creator
                    },
                    ipfs_hash: ipfs_hash.unwrap_or(p.ipfs_hash),
                    version: version.unwrap_or(p.version),
                    code_id: code_id.unwrap_or(p.code_id),
                    checksum: checksum.unwrap_or(p.checksum),
                })
            })?;

        let event = Event::new("vectis.plugin_registry.v1.MsgUpdatePlugin")
            .add_attribute("id", id.to_string())
            .add_attribute("plugin", format!("{plugin:?}"));

        Ok(Response::new().add_event(event))
    }

    #[msg(exec)]
    pub fn update_registry_fee(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        new_fee: Coin,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;
        ensure_eq!(
            deps.api.addr_humanize(&self.dao_addr.load(deps.storage)?)?,
            info.sender,
            ContractError::Unauthorized
        );

        self.registry_fee
            .update(deps.storage, |_| -> StdResult<Coin> { Ok(new_fee.clone()) })?;

        Ok(Response::default().add_event(
            Event::new("vectis.plugin_registry.v1.MsgUpdateRegistryFee")
                .add_attribute("amount", new_fee.amount)
                .add_attribute("denom", new_fee.denom),
        ))
    }

    #[msg(exec)]
    pub fn update_dao_addr(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        new_addr: String,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;
        ensure_eq!(
            deps.api.addr_humanize(&self.dao_addr.load(deps.storage)?)?,
            info.sender,
            ContractError::Unauthorized
        );

        self.dao_addr
            .update(deps.storage, |_| -> StdResult<CanonicalAddr> {
                deps.api
                    .addr_canonicalize(deps.api.addr_validate(new_addr.as_str())?.as_str())
            })?;

        Ok(Response::default().add_event(
            Event::new("vectis.plugin_registry.v1.MsgUpdateDaoAddr")
                .add_attribute("new_addr", new_addr),
        ))
    }

    #[msg(exec)]
    pub fn update_reviewer(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        reviewer: String,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;
        ensure_eq!(
            deps.api.addr_humanize(&self.dao_addr.load(deps.storage)?)?,
            info.sender,
            ContractError::Unauthorized
        );

        self.reviewer.save(
            deps.storage,
            &deps
                .api
                .addr_canonicalize(deps.api.addr_validate(&reviewer)?.as_str())?,
        )?;
        Ok(Response::default().add_event(
            Event::new("vectis.plugin_registry.v1.MsgUpdateReviewer")
                .add_attribute("reviewer", format!("{reviewer:?}")),
        ))
    }

    #[msg(query)]
    pub fn get_config(&self, ctx: (Deps, Env)) -> StdResult<ConfigResponse> {
        let (deps, ..) = ctx;
        Ok(ConfigResponse {
            registry_fee: self.registry_fee.load(deps.storage)?,
            dao_addr: deps
                .api
                .addr_humanize(&self.dao_addr.load(deps.storage)?)?
                .to_string(),
            reviewer: deps
                .api
                .addr_humanize(&self.reviewer.load(deps.storage)?)?
                .to_string(),
        })
    }

    #[msg(query)]
    pub fn get_plugins(
        &self,
        ctx: (Deps, Env),
        limit: Option<u32>,
        start_after: Option<u32>,
    ) -> StdResult<PluginsResponse> {
        let (deps, ..) = ctx;
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);

        let plugins = self
            .plugins
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(_, v)| v))
            .collect::<StdResult<Vec<Plugin>>>()?;

        Ok(PluginsResponse {
            plugins,
            total: self.total_plugins.load(deps.storage)?,
        })
    }

    #[msg(query)]
    pub fn get_plugin_by_id(&self, ctx: (Deps, Env), id: u64) -> StdResult<Option<Plugin>> {
        let (deps, ..) = ctx;
        self.plugins.may_load(deps.storage, id)
    }
}
