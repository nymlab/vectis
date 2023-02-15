use vectis_wallet::{ChainConfig, DaoConfig};

use crate::common::common::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RemoteChainSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// Admin Addr (for test we use admin, but it should be self)
    pub controller: Addr,
    // factory address
    pub factory: Addr,
    // remote tunnel address
    pub remote_tunnel: Addr,
}

impl RemoteChainSuite {
    /// In reality we instantiate remote_runnel contract
    /// Use remote_tunnel to instantiate_factory from DAO on dao-chain
    ///
    /// But we cannot access the ibc hooks so we do the factory first
    /// so that we can fill in the factory addr
    pub fn init() -> Result<RemoteChainSuite> {
        let genesis_funds = vec![coin(100000, "uremote")];
        let deployer = Addr::unchecked("deployer");
        let controller = Addr::unchecked(CONTROLLER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(
            deployer.clone(),
            controller.clone(),
            &[coin(50000, "uremote")],
        )?;

        let factory_id = app.store_code(contract_remote_factory());
        let proxy_id = app.store_code(contract_proxy());
        let multisig_id = app.store_code(contract_multisig());
        let remote_tunnel_id = app.store_code(contract_remote_tunnel());

        let factory_inst_msg = &InstantiateMsg {
            proxy_code_id: proxy_id,
            proxy_multisig_code_id: multisig_id,
            addr_prefix: "wasm".to_string(),
            wallet_fee: Coin {
                denom: "uremote".to_string(),
                amount: Uint128::new(WALLET_FEE),
            },
            claim_fee: Coin {
                amount: Uint128::one(),
                denom: "denom".to_string(),
            },
            govec_minter: None,
        };

        let factory = app
            .instantiate_contract(
                factory_id,
                deployer.clone(),
                factory_inst_msg,
                &[],
                "remote factory",
                Some(deployer.to_string()),
            )
            .unwrap();

        let remote_tunnel = app
            .instantiate_contract(
                remote_tunnel_id,
                deployer.clone(),
                &RTunnelInstanstiateMsg {
                    dao_config: DaoConfig {
                        addr: String::from("dao"),
                        dao_tunnel_port_id: String::from("dao-tunnel"),
                        connection_id: "dao-connection-id".to_string(),
                        dao_tunnel_channel: Some("dao-tunnel-channel".to_string()),
                    },
                    chain_config: ChainConfig {
                        remote_factory: None,
                        denom: "uremote".to_string(),
                    },
                    init_ibc_transfer_mod: None,
                },
                &[],
                "remote-tunnel",            // label: human readible name for contract
                Some(deployer.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        // update factory so dao is remote-tunnel
        app.execute_contract(
            deployer,
            factory.clone(),
            &WalletFactoryExecuteMsg::UpdateDao {
                addr: remote_tunnel.to_string(),
            },
            &[],
        )
        .unwrap();

        Ok(RemoteChainSuite {
            app,
            controller,
            factory,
            remote_tunnel,
        })
    }
    // Create wallet
    pub fn create_new_proxy_without_guardians(
        &mut self,
        controller: Addr,
        factory: Addr,
        proxy_initial_fund: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        self._create_new_proxy(
            controller,
            factory,
            proxy_initial_fund,
            guardians_multisig,
            vec![],
            native_tokens_amount,
        )
    }

    pub fn create_new_proxy(
        &mut self,
        controller: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let g1 = GUARD1.to_owned();
        let g2 = GUARD2.to_owned();
        self._create_new_proxy(
            controller,
            self.factory.clone(),
            proxy_initial_funds,
            guardians_multisig,
            vec![g1, g2],
            native_tokens_amount,
        )
    }

    fn _create_new_proxy(
        &mut self,
        controller: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        guardians: Vec<String>,
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let r = "relayer".to_owned();

        let create_wallet_msg = CreateWalletMsg {
            controller_addr: controller.to_string(),
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
                controller,
                factory,
                &execute,
                &[coin(native_tokens_amount, "uremote")],
            )
            .map_err(|err| anyhow!(err))?;

        let wasm_events: Vec<Event> = res
            .events
            .iter()
            .cloned()
            .filter(|ev| ev.ty.as_str() == "wasm")
            .collect();

        // This event
        let ev = wasm_events
            .iter()
            .find(|event| event.attributes.iter().any(|at| at.key == "proxy_address"));

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
            controller_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            signature: Binary(sig.serialize_compact().to_vec()),
            nonce,
        }
    }

    pub fn update_proxy_code_id(&mut self, new_code_id: u64, factory: Addr) -> Result<AppResponse> {
        self.app
            .execute_contract(
                self.remote_tunnel.clone(),
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
                self.remote_tunnel.clone(),
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

    pub fn query_balance(&self, addr: &Addr) -> Result<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), "uremote")?)
    }

    pub fn claim_expiration(&self) -> u64 {
        GOVEC_CLAIM_DURATION_DAY_MUL * 24 * 60 * 60
    }
}
