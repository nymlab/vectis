use cosmwasm_std::{coin, to_binary, Coin, CosmosMsg, Uint128, WasmMsg};
use osmosis_test_tube::{Account, OsmosisTestApp, SigningAccount};

use vectis_factory::contract::sv::InstantiateMsg as FactoryInstMsg;
use vectis_plugin_registry::contract::sv::InstantiateMsg as PluginRegInstMsg;

use vectis_wallet::{
    interface::registry_management_trait::sv as registry_management_trait,
    types::{
        authenticator::{AuthenticatorType, EmptyInstantiateMsg},
        factory::{AuthenticatorInstInfo, ChainConnection, WalletFactoryInstantiateMsg},
        plugin::PluginsResponse,
        plugin_registry::SubscriptionTier,
        state::VectisActors,
    },
};

use crate::{
    constants::*,
    test_tube::util::{contract::Contract, vectis_committee},
};
use cw3_flex_multisig::msg::{ExecuteMsg as cw3flexExecMsg, InstantiateMsg as cw3flexInstMsg};
use cw4::Member;
use cw4_group::msg::InstantiateMsg as cw4InstMsg;

/// This is initialised all the contracts
/// - factory: creates, upgrade proxy and store govec claim list
/// - plugin registry
/// - deployer: a multisig with some additional states
/// - plugin committee: a multisig
pub struct HubChainSuite<'a> {
    pub app: &'a OsmosisTestApp,
    // factory address
    pub factory: String,
    // Webauthn address
    pub webauthn: String,
    // Vectis committee: deployer address: cw3_flex_multisig
    pub deployer: String,
    // deployer group cw4
    pub deployer_group: String,
    // PluginRegistry addr
    pub plugin_registry: String,
    // accounts
    pub accounts: Vec<SigningAccount>,
    // test plugins
    pub test_contracts: TestContracts,
}

impl<'a> HubChainSuite<'a> {
    pub fn init(app: &'a OsmosisTestApp) -> Self {
        // Create some accounts for testing
        let accounts = app
            .init_accounts(&[Coin::new(1000000000000000u128, "uosmo")], 10)
            .unwrap();

        // Deploy cw4group for vectis committee and plugin committee
        let mgmt_group = Contract::deploy(
            app,
            CW4_CODE_PATH,
            &cw4InstMsg {
                admin: None,
                members: vec![Member {
                    addr: accounts[ICOMMITTEE].address(),
                    weight: 1,
                }],
            },
            &accounts[IDEPLOYER],
        )
        .unwrap();

        // Deploy the cw3 Flex for deploying factory
        let deployer = Contract::deploy(
            app,
            CW3FLEX_CODE_PATH,
            &cw3flexInstMsg {
                group_addr: mgmt_group.contract_addr.to_string(),
                threshold: cw_utils::Threshold::AbsoluteCount { weight: 1 },
                max_voting_period: cw_utils::Duration::Time(100),
                executor: None,
                proposal_deposit: None,
            },
            &accounts[0],
        )
        .unwrap();

        // Store code for proxy and authenticator
        let test_pre_tx_code_id = Contract::store_code(app, &PRE_TX_CODE_PATH, &accounts[IDEPLOYER]);
        let test_post_tx_code_id =
            Contract::store_code(app, &POST_TX_CODE_PATH, &accounts[IDEPLOYER]);
        let test_plugin_exec_code_id =
            Contract::store_code(app, &PLUGIN_EXEC_CODE_PATH, &accounts[IDEPLOYER]);

        let proxy_code_id = Contract::store_code(app, &PROXY_CODE_PATH, &accounts[IDEPLOYER]);
        let auth_code_id = Contract::store_code(app, &AUTH_CODE_PATH, &accounts[IDEPLOYER]);
        let factory_code_id = Contract::store_code(app, &FACTORY_CODE_PATH, &accounts[IDEPLOYER]);
        let plugin_reg_code_id =
            Contract::store_code(app, &REGISTRY_CODE_PATH, &accounts[IDEPLOYER]);
        let proxy_migration_code_id =
            Contract::store_code(app, PROXY_MIGRATION_CODE_PATH, &accounts[IDEPLOYER]);

        let factory_inst_msg = WalletFactoryInstantiateMsg {
            default_proxy_code_id: proxy_code_id,
            supported_proxies: vec![
                (proxy_code_id, VECTIS_VERSION.into()),
                (proxy_migration_code_id, PROXY_MIGRATE_VERSION.into()),
                (test_pre_tx_code_id, "should-fail-v2.0.0-test".into()),
            ],
            wallet_fee: Coin {
                denom: DENOM.to_string(),
                amount: Uint128::new(WALLET_FEE),
            },
            authenticators: Some(vec![AuthenticatorInstInfo {
                ty: AuthenticatorType::Webauthn,
                code_id: auth_code_id,
                inst_msg: to_binary(&EmptyInstantiateMsg {}).unwrap(),
            }]),
            supported_chains: Some(vec![
                (
                    IBC_CHAIN_NAME.into(),
                    ChainConnection::IBC(NON_IBC_CHAIN_CONN.into()),
                ),
                (
                    NON_IBC_CHAIN_NAME.into(),
                    ChainConnection::Other(NON_IBC_CHAIN_CONN.into()),
                ),
            ]),
        };

        // ===========================================================
        //
        // Propose and execute deploy of factory
        //
        // ===========================================================

        let deploy_factory_proposal = cw3flexExecMsg::Propose {
            title: "Deploy Vectis Factory".into(),
            description: "".into(),
            msgs: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(deployer.contract_addr.clone()),
                code_id: factory_code_id,
                msg: to_binary(&FactoryInstMsg {
                    msg: factory_inst_msg,
                })
                .unwrap(),
                funds: vec![],
                label: "Vectis Factory".into(),
            })],
            latest: None,
        };
        let exeucte_factory_proposal = cw3flexExecMsg::Execute { proposal_id: 1 };

        deployer
            .execute(&deploy_factory_proposal, &[], &accounts[ICOMMITTEE])
            .unwrap();
        let res = deployer
            .execute(&exeucte_factory_proposal, &[], &accounts[ICOMMITTEE])
            .unwrap();

        let mut events = res.events.into_iter();

        let factory: String = events
            .find(|x| x.ty == "wasm-vectis.factory.v1")
            .unwrap()
            .attributes
            .into_iter()
            .find(|x| x.key == "_contract_address")
            .unwrap()
            .value;

        let webauthn: String = events
            .find(|x| x.ty == "wasm-vectis.webauthn.v1")
            .unwrap()
            .attributes
            .into_iter()
            .find(|x| x.key == "_contract_address")
            .unwrap()
            .value;

        // ===========================================================
        //
        // Propose and execute deploy of plugin-registry
        //
        // ===========================================================
        let plugin_reg_inst_msg = PluginRegInstMsg {
            subscription_tiers: vec![
                (SubscriptionTier::Free, tier_0()),
                (SubscriptionTier::L1, tier_1()),
            ],
            registry_fee: coin(REGISTRY_FEE, DENOM),
        };
        let deploy_reg_proposal = cw3flexExecMsg::Propose {
            title: "Deploy Vectis Registry".into(),
            description: "".into(),
            msgs: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
                admin: Some(deployer.contract_addr.clone()),
                code_id: plugin_reg_code_id,
                msg: to_binary(&plugin_reg_inst_msg).unwrap(),
                funds: vec![],
                label: "Vectis Registry".into(),
            })],
            latest: None,
        };
        let exeucte_plugin_proposal = cw3flexExecMsg::Execute { proposal_id: 2 };
        deployer
            .execute(&deploy_reg_proposal, &[], &accounts[ICOMMITTEE])
            .unwrap();
        let res = deployer
            .execute(&exeucte_plugin_proposal, &[], &accounts[ICOMMITTEE])
            .unwrap();

        let plugin_registry: String = res
            .events
            .into_iter()
            .find(|x| x.ty == "wasm-vectis.plugin_registry.v1")
            .unwrap()
            .attributes
            .into_iter()
            .find(|x| x.key == "_contract_address")
            .unwrap()
            .value;

        // ===============================================================
        //
        // Update Deployer with ITEMS: Factory, PluginReg, PluginCommitte
        //
        // ==============================================================

        // Factory
        deployer
            .execute(
                &add_item_prop_msg(
                    deployer.contract_addr.clone(),
                    VectisActors::Factory,
                    factory.clone(),
                ),
                &[],
                &accounts[ICOMMITTEE],
            )
            .unwrap();
        deployer
            .execute(
                &cw3flexExecMsg::Execute { proposal_id: 3 },
                &[],
                &accounts[ICOMMITTEE],
            )
            .unwrap();

        // PluginReg
        deployer
            .execute(
                &add_item_prop_msg(
                    deployer.contract_addr.clone(),
                    VectisActors::PluginRegistry,
                    plugin_registry.clone(),
                ),
                &[],
                &accounts[ICOMMITTEE],
            )
            .unwrap();
        deployer
            .execute(
                &cw3flexExecMsg::Execute { proposal_id: 4 },
                &[],
                &accounts[ICOMMITTEE],
            )
            .unwrap();

        Self {
            app,
            deployer_group: mgmt_group.contract_addr,
            deployer: deployer.contract_addr,
            factory,
            webauthn,
            plugin_registry,
            accounts,
            test_contracts: TestContracts {
                pre_tx: (test_pre_tx_code_id, &PRE_TX_HASH, 0),
                post_tx: (test_post_tx_code_id, &POST_TX_HASH, 0),
                exec: (test_plugin_exec_code_id, &PLUGIN_EXEC_HASH, 0),
                proxy_migrate: (proxy_migration_code_id, PROXY_MIGRATION_HASH),
            },
        }
    }
}

impl HubChainSuite<'_> {
    pub fn register_plugins(&mut self) {
        vectis_committee::execute(
            self.app,
            self.deployer.clone(),
            self.plugin_registry.clone(),
            &registry_management_trait::ExecMsg::RegisterPlugin {
                code_data: test_plugin_code_data(
                    self.test_contracts.pre_tx.0,
                    self.test_contracts.pre_tx.1,
                ),
                metadata_data: test_plugin_metadata(),
            },
            &[coin(REGISTRY_FEE, "uosmo")],
            &self.accounts[ICOMMITTEE],
        )
        .unwrap();

        vectis_committee::execute(
            self.app,
            self.deployer.clone(),
            self.plugin_registry.clone(),
            &registry_management_trait::ExecMsg::RegisterPlugin {
                code_data: test_plugin_code_data(
                    self.test_contracts.post_tx.0,
                    self.test_contracts.post_tx.1,
                ),
                metadata_data: test_plugin_metadata(),
            },
            &[coin(REGISTRY_FEE, "uosmo")],
            &self.accounts[ICOMMITTEE],
        )
        .unwrap();

        vectis_committee::execute(
            self.app,
            self.deployer.clone(),
            self.plugin_registry.clone(),
            &registry_management_trait::ExecMsg::RegisterPlugin {
                code_data: test_plugin_code_data(
                    self.test_contracts.exec.0,
                    self.test_contracts.exec.1,
                ),
                metadata_data: test_plugin_metadata(),
            },
            &[coin(REGISTRY_FEE, "uosmo")],
            &self.accounts[ICOMMITTEE],
        )
        .unwrap();

        let registry = Contract::from_addr(self.app, self.plugin_registry.clone());

        let plugins: PluginsResponse = registry
            .query(&registry_management_trait::QueryMsg::GetPlugins {
                limit: None,
                start_after: None,
            })
            .unwrap();

        assert_eq!(plugins.total, 3);

        self.test_contracts.pre_tx.2 = 1;
        self.test_contracts.post_tx.2 = 2;
        self.test_contracts.exec.2 = 3;
    }
}

pub fn add_item_prop_msg(deployer: String, key: VectisActors, value: String) -> cw3flexExecMsg {
    cw3flexExecMsg::Propose {
        title: "Add_Item".into(),
        description: key.to_string(),
        msgs: vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deployer,
            msg: to_binary(&cw3flexExecMsg::UpdateItem {
                key: format!("{key}"),
                value: value.to_string(),
            })
            .unwrap(),
            funds: vec![],
        })],
        latest: None,
    }
}
