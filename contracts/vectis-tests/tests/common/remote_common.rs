use vectis_wallet::{ChainConfig, DaoConfig};

use crate::common::common::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RemoteChainSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// Admin Addr (for test we use admin, but it should be self)
    pub user: Addr,
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
        let user = Addr::unchecked(USER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(deployer.clone(), user.clone(), &[coin(50000, "uremote")])?;

        let factory_id = app.store_code(contract_factory());
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
            deployer.clone(),
            factory.clone(),
            &WalletFactoryExecuteMsg::UpdateDao {
                addr: remote_tunnel.to_string(),
            },
            &[],
        )
        .unwrap();

        Ok(RemoteChainSuite {
            app,
            user,
            factory,
            remote_tunnel,
        })
    }

    pub fn create_new_proxy(
        &mut self,
        mut proxy_initial_funds: Vec<Coin>,
        native_tokens_amount: u128,
    ) -> Result<Addr> {
        let create_wallet_msg = CreateWalletMsg {
            user_addr: self.user.to_string(),
            guardians: Guardians {
                addresses: vec![],
                guardians_multisig: None,
            },
            relayers: vec![],
            proxy_initial_funds: proxy_initial_funds.clone(),
            label: "initial label".to_string(),
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };
        proxy_initial_funds.push(coin(native_tokens_amount, "uremote"));

        let res = self
            .app
            .execute_contract(
                self.user.clone(),
                self.factory.clone(),
                &execute,
                &proxy_initial_funds,
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
}
