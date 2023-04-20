use cosmwasm_schema::cw_serde;
use cw2::set_contract_version;
use cw_storage_plus::{Bound, Item, Map};
use sylvia::{contract, schemars};

use cosmwasm_std::{
    ensure_eq, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env, Event,
    MessageInfo, Order, Response, StdResult, SubMsg, Uint128, WasmMsg,
};

use vectis_wallet::{get_items_from_deployer, VectisActors, DEFAULT_LIMIT, DEPLOYER, MAX_LIMIT};

use crate::{
    error::ContractError,
    responses::{ConfigResponse, PluginsResponse},
    INSTALL_REPLY,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct Fees {
    install: Coin,
    registry: Coin,
}

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
    pub(crate) plugins: Map<'a, u64, Plugin>,
    pub install_fee: Item<'a, Coin>,
}

#[contract]
impl PluginRegistry<'_> {
    pub const fn new() -> Self {
        Self {
            total_plugins: Item::new("total_plugins"),
            registry_fee: Item::new("registry_fee"),
            plugins: Map::new("plugins"),
            install_fee: Item::new("install_fee"),
        }
    }

    #[msg(exec)]
    pub fn proxy_install_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        id: u64,
        instantiate_msg: Binary,
    ) -> Result<Response, ContractError> {
        let (deps, _env, mut info) = ctx;
        let plugin = self.plugins.load(deps.storage, id)?;
        // ensure proxy sent enough fee for registry
        let install_fee = self.install_fee.load(deps.storage)?;
        self.sub_required_fee(&mut info.funds, &install_fee)?;

        // if fund amount is 0, instantiate call fails
        let mut validated_funds = info.funds.to_vec();
        if let Some(index) = validated_funds
            .to_owned()
            .iter()
            .position(|x| x.amount == Uint128::zero())
        {
            validated_funds.remove(index);
        }

        // TODO: extra check can be done on cosmwasm_std v1.2
        // let code_info = deps.querier.query_wasm_code_info(plugin.code_id)?;
        // if code_info.checksum != plugin.checksum {
        //     return Err(ContractError::ChecksumVerificationFailed);
        // }

        let sub_msg = SubMsg::reply_always(
            WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: plugin.code_id,
                msg: instantiate_msg,
                funds: validated_funds,
                label: format!("{}-{}", plugin.name, plugin.version),
            },
            INSTALL_REPLY,
        );

        let event = Event::new("vectis.plugin_registry.v1.MsgInstallPluginRequested")
            .add_attribute("plugin_id", id.to_string())
            .add_attribute("wallet", info.sender);
        // Send funds to the Deployer
        if !install_fee.amount.is_zero() {
            let msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&DEPLOYER.load(deps.storage)?)?
                    .to_string(),
                amount: vec![install_fee],
            });
            Ok(Response::new()
                .add_submessage(sub_msg)
                .add_message(msg)
                .add_event(event))
        } else {
            Ok(Response::new().add_submessage(sub_msg).add_event(event))
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        registry_fee: Coin,
        install_fee: Coin,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        self.total_plugins.save(deps.storage, &0u64)?;
        self.registry_fee.save(deps.storage, &registry_fee)?;
        self.install_fee.save(deps.storage, &install_fee)?;
        DEPLOYER.save(
            deps.storage,
            &deps.api.addr_canonicalize(info.sender.as_str())?,
        )?;
        Ok(Response::default())
    }

    fn ensure_is_reviewer(&self, deps: Deps, sender: &str) -> Result<(), ContractError> {
        let reviewer = get_items_from_deployer(deps, VectisActors::PluginCommittee)?;
        if reviewer != sender {
            Err(ContractError::Unauthorized)
        } else {
            Ok(())
        }
    }

    /// Subtracts the required fees defined in the registry to be sent to DEPLOYER treasury
    /// and return the remaining funds sent to this contract - presumably for the plugin
    fn sub_required_fee<'a>(
        &'a self,
        provided: &'a mut [Coin],
        required: &Coin,
    ) -> Result<(), ContractError> {
        if required.amount != Uint128::zero() {
            let fund = provided
                .iter_mut()
                .find(|c| c.denom == required.denom)
                .ok_or(ContractError::InsufficientFee(
                    required.amount,
                    Uint128::zero(),
                ))?;
            fund.amount = fund
                .amount
                .checked_sub(required.amount)
                .map_err(|_| ContractError::InsufficientFee(required.amount, fund.amount))?;
        }

        Ok(())
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
        let (deps, _env, mut info) = ctx;

        // Check if the caller has enough funds to pay the fee
        let registry_fee = self.registry_fee.load(deps.storage)?;
        self.sub_required_fee(&mut info.funds, &registry_fee)?;

        // Check if the caller is a reviewer
        self.ensure_is_reviewer(deps.as_ref(), info.sender.as_str())?;

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

        // Send funds to the Deployer if required
        if registry_fee.amount != Uint128::zero() {
            let msg = CosmosMsg::Bank(BankMsg::Send {
                to_address: deps
                    .api
                    .addr_humanize(&DEPLOYER.load(deps.storage)?)?
                    .to_string(),
                amount: vec![registry_fee],
            });
            Ok(Response::new()
                .add_event(Event::new("vectis.plugin_registry.v1.MsgInstantiate"))
                .add_message(msg))
        } else {
            Ok(Response::new().add_event(Event::new("vectis.plugin_registry.v1.MsgInstantiate")))
        }
    }

    #[msg(exec)]
    pub fn unregister_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        id: u64,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;

        // Check if the caller is a reviewer
        self.ensure_is_reviewer(deps.as_ref(), info.sender.as_str())?;

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

        // Check if the caller is a reviewer
        self.ensure_is_reviewer(deps.as_ref(), info.sender.as_str())?;

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
            deps.api.addr_humanize(&DEPLOYER.load(deps.storage)?)?,
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
    pub fn update_deployer_addr(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        new_addr: String,
    ) -> Result<Response, ContractError> {
        let (deps, _env, info) = ctx;
        ensure_eq!(
            deps.api.addr_humanize(&DEPLOYER.load(deps.storage)?)?,
            info.sender,
            ContractError::Unauthorized
        );

        DEPLOYER.update(deps.storage, |_| -> StdResult<CanonicalAddr> {
            deps.api
                .addr_canonicalize(deps.api.addr_validate(new_addr.as_str())?.as_str())
        })?;

        Ok(Response::default().add_event(
            Event::new("vectis.plugin_registry.v1.MsgUpdateDaoAddr")
                .add_attribute("new_addr", new_addr),
        ))
    }

    #[msg(query)]
    pub fn get_config(&self, ctx: (Deps, Env)) -> StdResult<ConfigResponse> {
        let (deps, ..) = ctx;
        Ok(ConfigResponse {
            registry_fee: self.registry_fee.load(deps.storage)?,
            deployer_addr: deps
                .api
                .addr_humanize(&DEPLOYER.load(deps.storage)?)?
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

    #[msg(query)]
    pub fn get_fees(&self, ctx: (Deps, Env)) -> StdResult<Fees> {
        let (deps, ..) = ctx;
        let install = self.install_fee.load(deps.storage)?;
        let registry = self.registry_fee.load(deps.storage)?;
        Ok(Fees { install, registry })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::coin;

    #[test]
    fn sub_required_fee_works() {
        let registry_fee = coin(10u128, "ucosm");
        let mut provided = vec![coin(20u128, "ucosm")];
        let registry = PluginRegistry::new();
        registry
            .sub_required_fee(&mut provided, &registry_fee)
            .unwrap();
        assert_eq!(provided[0].amount, Uint128::new(10u128));
    }

    #[test]
    fn zero_fees_required_fee_works() {
        let registry_fee = coin(0u128, "ucosm");
        let mut provided = vec![coin(20u128, "ucosm")];
        let registry = PluginRegistry::new();
        registry
            .sub_required_fee(&mut provided, &registry_fee)
            .unwrap();
        assert_eq!(provided[0].amount, Uint128::new(20u128));
    }

    #[test]
    fn sub_required_fee_fails_when_no_denom_found() {
        let no_denom_registry_fee = coin(10u128, "ujunox");
        let mut provided = vec![coin(20u128, "ucosm")];
        let registry = PluginRegistry::new();
        let err = registry
            .sub_required_fee(&mut provided, &no_denom_registry_fee)
            .unwrap_err();
        assert_eq!(
            err,
            ContractError::InsufficientFee(Uint128::new(10), Uint128::zero())
        )
    }

    #[test]
    fn sub_required_fee_fails_insuffient_fees() {
        let too_much_registry_fee = coin(100u128, "ucosm");
        let mut provided = vec![coin(20u128, "ucosm")];
        let registry = PluginRegistry::new();
        let err = registry
            .sub_required_fee(&mut provided, &too_much_registry_fee)
            .unwrap_err();
        assert_eq!(
            err,
            ContractError::InsufficientFee(Uint128::new(100), Uint128::new(20))
        )
    }
}
