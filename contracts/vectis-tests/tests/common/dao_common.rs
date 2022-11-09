pub use crate::common::common::*;

use voting::Threshold;

/// DaoChainSuite
///
/// This is initialised all the contracts that will be interacting with on the dao-chain
/// - dao: cw-core
/// - proposal single: one of the proposal modules (direct interaction with proxy)
/// - cw20_staked_balance_voting (voting power): queried by dao
/// - cw20_stake: where staked govec are held, reports to cw20_staked_balance_voting (direct interaction with proxy)
/// - factory: creates, upgrade proxy and store govec claim list
/// - dao_tunnel: ibc enabled to communicate with remote_tunnel
/// - govec (whitelist-cw20): (direct interaction with proxy)
#[derive(Derivative)]
#[derivative(Debug)]
pub struct DaoChainSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    // The account that deploys everything and remove itself
    pub deployer: Addr,
    // User
    pub user: Addr,
    // govec address
    pub govec: Addr,
    // factory address
    pub factory: Addr,
    // dao_tunnel address
    pub dao_tunnel: Addr,
    // dao address
    pub dao: Addr,
    // proposal module address
    pub proposal: Addr,
    // cw20_stake addr
    pub cw20_stake: Addr,
    // Voting addr
    pub voting: Addr,
}

impl DaoChainSuite {
    /// Instantiate factory contract with
    /// - no initial funds on the factory
    /// - default WALLET_FEE
    ///
    pub fn init() -> Result<DaoChainSuite> {
        let genesis_funds = vec![coin(100000, "ucosm")];
        let deployer = Addr::unchecked("deployer");
        let user = Addr::unchecked(USER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        let for_user = vec![coin(50000, "ucosm")];
        app.send_tokens(deployer.clone(), user, &for_user)?;

        let dao_id = app.store_code(contract_dao());
        let vote_id = app.store_code(contract_vote());
        let proposal_id = app.store_code(contract_proposal());
        let stake_id = app.store_code(contract_stake());
        let factory_id = app.store_code(contract_factory());
        let proxy_id = app.store_code(contract_proxy());
        let proxy_multisig_id = app.store_code(contract_multisig());
        let govec_id = app.store_code(contract_govec());
        let dao_tunnel_id = app.store_code(contract_dao_tunnel());

        let govec = app
            .instantiate_contract(
                govec_id,
                deployer.clone(),
                &GovecInstantiateMsg {
                    name: String::from("govec"),
                    symbol: String::from("gov"),
                    initial_balances: vec![],
                    staking_addr: None,
                    marketing: None,
                    mint_cap: None,
                    factory: None,
                    dao_tunnel: None,
                },
                &[],
                "govec",
                Some(deployer.to_string()),
            )
            .unwrap();

        let dao = app
            .instantiate_contract(
                dao_id,
                deployer.clone(),
                &DaoInstMsg {
                    // sets this so we can execute some messages directly
                    admin: Some(deployer.to_string()),
                    name: String::from("VectisDAO"),
                    description: String::from("Wallets on steriod"),
                    image_url: None,
                    automatically_add_cw20s: true,
                    automatically_add_cw721s: true,
                    voting_module_instantiate_info: ModuleInstantiateInfo {
                        code_id: vote_id,
                        msg: to_binary(&VoteInstMsg {
                            token_info: TokenInfo::Existing {
                                address: govec.to_string(),
                                staking_contract: StakingInfo::New {
                                    staking_code_id: stake_id,
                                    unstaking_duration: None,
                                },
                            },
                            active_threshold: None,
                        })
                        .unwrap(),
                        admin: Admin::CoreContract {},
                        label: String::from("Vectis Vote Module"),
                    },
                    proposal_modules_instantiate_info: vec![ModuleInstantiateInfo {
                        code_id: proposal_id,
                        msg: to_binary(&PropInstMsg {
                            threshold: Threshold::AbsoluteCount {
                                threshold: Uint128::new(1u128),
                            },
                            max_voting_period: Duration::Height(99u64),
                            min_voting_period: None,
                            only_members_execute: false,
                            allow_revoting: false,
                            deposit_info: Some(DepositInfo {
                                token: DepositToken::Token {
                                    address: govec.to_string(),
                                },
                                deposit: Uint128::one(),
                                refund_failed_proposals: true,
                            }),
                        })
                        .unwrap(),
                        admin: Admin::CoreContract {},
                        label: String::from("Vectis Dao Prop single"),
                    }],
                    initial_items: None,
                },
                &[],
                "dao",
                Some(deployer.to_string()),
            )
            .unwrap();

        let prop_list: Vec<Addr> = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: dao.to_string(),
                msg: to_binary(&DaoQueryMsg::ProposalModules {
                    start_at: None,
                    limit: None,
                })
                .unwrap(),
            }))
            .unwrap();

        let voting: Addr = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: dao.to_string(),
                msg: to_binary(&DaoQueryMsg::VotingModule {}).unwrap(),
            }))
            .unwrap();

        let cw20_stake: Addr = app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: voting.to_string(),
                msg: to_binary(&VoteQueryMsg::StakingContract {}).unwrap(),
            }))
            .unwrap();

        let factory = app
            .instantiate_contract(
                factory_id,
                deployer.clone(),
                &InstantiateMsg {
                    proxy_code_id: proxy_id,
                    proxy_multisig_code_id: proxy_multisig_id,
                    addr_prefix: "wasm".to_string(),
                    wallet_fee: Coin {
                        denom: "ucosm".to_string(),
                        amount: Uint128::new(WALLET_FEE),
                    },
                    govec_minter: Some(govec.to_string()),
                },
                &[],
                "wallet-factory", // label: human readible name for contract
                Some(deployer.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        let dao_tunnel = app
            .instantiate_contract(
                dao_tunnel_id,
                deployer.clone(),
                &DTunnelInstanstiateMsg {
                    govec_minter: govec.to_string(),
                    init_remote_tunnels: None,
                },
                &[],
                "dao-tunnel",               // label: human readible name for contract
                Some(deployer.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        // Give Govec the correct contractg addresses
        app.execute_contract(
            deployer.clone(),
            govec.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::Factory(factory.to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        app.execute_contract(
            deployer.clone(),
            govec.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::DaoTunnel(dao_tunnel.to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        app.execute_contract(
            deployer.clone(),
            govec.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::Staking(cw20_stake.to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        app.execute_contract(
            deployer.clone(),
            govec.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::Proposal(prop_list[0].to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        Ok(DaoChainSuite {
            user: Addr::unchecked(USER_ADDR),
            app,
            deployer,
            govec,
            factory,
            dao_tunnel,
            dao,
            proposal: prop_list[0].clone(),
            cw20_stake,
            voting,
        })
    }

    pub fn create_new_proxy_without_guardians(
        &mut self,
        user: Addr,
        factory: Addr,
        proxy_initial_fund: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        self._create_new_proxy(
            user,
            factory,
            proxy_initial_fund,
            guardians_multisig,
            vec![],
            native_tokens_amount,
        )
    }

    pub fn create_new_proxy(
        &mut self,
        user: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let g1 = GUARD1.to_owned();
        let g2 = GUARD2.to_owned();
        self._create_new_proxy(
            user,
            factory,
            proxy_initial_funds,
            guardians_multisig,
            vec![g1, g2],
            native_tokens_amount,
        )
    }

    fn _create_new_proxy(
        &mut self,
        user: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        guardians: Vec<String>,
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let r = "relayer".to_owned();

        let create_wallet_msg = CreateWalletMsg {
            user_addr: user.to_string(),
            guardians: Guardians {
                addresses: guardians,
                guardians_multisig,
            },
            relayers: vec![r],
            proxy_initial_funds,
            label: "initial label".to_string(),
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };

        let res = self
            .app
            .execute_contract(
                user,
                factory,
                &execute,
                &[coin(native_tokens_amount, "ucosm")],
            )
            .map_err(|err| anyhow!(err))?;

        let wasm_events: Vec<Event> = res
            .events
            .iter()
            .cloned()
            .filter(|ev| ev.ty.as_str() == "wasm")
            .collect();

        // This event
        let ev = wasm_events.iter().find(|event| {
            event
                .attributes
                .iter()
                .find(|at| at.key == "proxy_address")
                .is_some()
        });

        let proxy = &ev.unwrap().attributes[2].value;

        Ok(Addr::unchecked(proxy))
    }

    pub fn create_relay_transaction(
        &mut self,
        signer_sk: &[u8; 32],
        cosmos_msg: CosmosMsg,
        nonce: u64,
    ) -> RelayTransaction {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(signer_sk).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let msg_bytes = to_binary(&cosmos_msg).unwrap();

        let message_with_nonce = Message::from_hashed_data::<sha256::Hash>(
            &msg_bytes
                .iter()
                .chain(&nonce.to_be_bytes())
                .copied()
                .collect::<Vec<u8>>(),
        );
        let sig = secp.sign(&message_with_nonce, &secret_key);

        RelayTransaction {
            message: Binary(msg_bytes.to_vec()),
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        }
    }

    pub fn update_proxy_code_id(&mut self, new_code_id: u64, factory: Addr) -> Result<AppResponse> {
        self.app
            .execute_contract(
                self.deployer.clone(),
                factory,
                &FactoryExecuteMsg::UpdateCodeId {
                    ty: CodeIdType::Proxy,
                    new_code_id,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn update_proxy_multisig_code_id(
        &mut self,
        new_code_id: u64,
        factory: Addr,
    ) -> Result<AppResponse> {
        self.app
            .execute_contract(
                self.deployer.clone(),
                factory,
                &FactoryExecuteMsg::UpdateCodeId {
                    ty: CodeIdType::Multisig,
                    new_code_id,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn create_guardians_request_and_update_guardians(
        &mut self,
        sender: &Addr,
        wallet_address: &Addr,
        guardians: Guardians,
        new_multisig_code_id: Option<u64>,
    ) -> Result<AppResponse> {
        let request = GuardiansUpdateMsg {
            guardians,
            new_multisig_code_id,
        };

        let request_update_guardians_message: ProxyExecuteMsg =
            ProxyExecuteMsg::RequestUpdateGuardians {
                request: Some(request),
            };

        self.app
            .execute_contract(
                sender.clone(),
                wallet_address.clone(),
                &request_update_guardians_message,
                &[],
            )
            .unwrap();

        self.fast_forward_block_time(90000);

        let update_guardians_message: ProxyExecuteMsg = ProxyExecuteMsg::UpdateGuardians {};

        let r = self
            .app
            .execute_contract(
                sender.clone(),
                wallet_address.clone(),
                &update_guardians_message,
                &[],
            )
            .unwrap();

        Ok(r)
    }

    pub fn fast_forward_block_time(&mut self, forward_time_sec: u64) {
        let block = self.app.block_info();

        let mock_block = BlockInfo {
            height: block.height + 10,
            chain_id: block.chain_id,
            time: block.time.plus_seconds(forward_time_sec),
        };

        self.app.set_block(mock_block);
    }

    pub fn query_unclaimed_govec_wallets(
        &self,
        contract_addr: &Addr,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<UnclaimedWalletList, StdError> {
        let r = self
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&FactoryQueryMsg::UnclaimedGovecWallets { start_after, limit })
                    .unwrap(),
            }))
            .unwrap();
        Ok(r)
    }

    pub fn query_proxy_govec_claim_expiration(
        &self,
        contract_addr: &Addr,
        proxy: &Addr,
    ) -> Result<Option<Expiration>, StdError> {
        let r = self
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&FactoryQueryMsg::ClaimExpiration {
                    wallet: proxy.to_string(),
                })
                .unwrap(),
            }))
            .unwrap();
        Ok(r)
    }

    pub fn query_wallet_info<R>(&self, contract_addr: &Addr) -> Result<R, StdError>
    where
        R: DeserializeOwned,
    {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ProxyQueryMsg::Info {}).unwrap(),
        }))
    }

    pub fn query_guardians_request(
        &self,
        contract_addr: &Addr,
    ) -> Result<Option<GuardiansUpdateRequest>, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&ProxyQueryMsg::GuardiansUpdateRequest {}).unwrap(),
        }))
    }

    pub fn query_multisig_voters(
        &self,
        contract_addr: &Addr,
    ) -> Result<VoterListResponse, StdError> {
        self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&MultiSigQueryMsg::ListVoters {
                start_after: None,
                limit: None,
            })
            .unwrap(),
        }))
    }

    pub fn query_balance(&self, addr: &Addr, denom: String) -> Result<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), denom)?)
    }

    pub fn query_govec_balance(&self, proxy: &Addr) -> Result<BalanceResponse, StdError> {
        let r = self
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: self.govec.to_string(),
                msg: to_binary(&GovecQueryMsg::Balance {
                    address: proxy.to_string(),
                })?,
            }))?;
        Ok(r)
    }

    pub fn claim_expiration(&self) -> u64 {
        GOVEC_CLAIM_DURATION_DAY_MUL * 24 * 60 * 60
    }
}
