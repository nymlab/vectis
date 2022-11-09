use crate::common::common::*;

pub struct RemoteChainSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// Admin Addr (for test we use admin, but it should be self)
    pub owner: Addr,
    /// ID of stored code for factory
    pub sc_factory_id: u64,
    // ID of stored code for proxy
    pub sc_proxy_id: u64,
    // ID of stored code for proxy multisig
    pub sc_proxy_multisig_code_id: u64,
    // ID of dao_tunnel contract
    pub remote_tunnel_id: u64,
    // factory address
    pub factory_addr: Addr,
    // remote tunnel address
    pub remote_tunnel_addr: Addr,
}

impl RemoteChainSuite {
    /// Instantiate factory contract with
    /// - no initial funds on the factory
    /// - default WALLET_FEE
    /// - code ids from RemoteChainSuite
    pub fn init() -> Result<RemoteChainSuite> {
        let genesis_funds = vec![coin(100000, "uremote")];
        let owner = Addr::unchecked("owner");
        let user = Addr::unchecked(USER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router.bank.unwrap();
        });
        let for_user = vec![coin(50000, "uremote")];
        app.send_tokens(owner.clone(), user, &for_user)?;

        let sc_factory_id = app.store_code(contract_factory());
        let sc_proxy_id = app.store_code(contract_proxy());
        let sc_proxy_multisig_code_id = app.store_code(contract_multisig());
        let remote_tunnel_id = app.store_code(contract_remote_tunnel());

        let govec_addr = app
            .instantiate_contract(
                govec_id,
                owner.clone(),
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
                Some(owner.to_string()),
            )
            .unwrap();

        let factory_addr = app
            .instantiate_contract(
                sc_factory_id,
                owner.clone(),
                &InstantiateMsg {
                    proxy_code_id: sc_proxy_id,
                    proxy_multisig_code_id: sc_proxy_multisig_code_id,
                    addr_prefix: "wasm".to_string(),
                    wallet_fee: Coin {
                        denom: "ucosm".to_string(),
                        amount: Uint128::new(WALLET_FEE),
                    },
                    govec_minter: Some(govec_addr.to_string()),
                },
                &[],
                "wallet-factory",        // label: human readible name for contract
                Some(owner.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        let dao_tunnel_addr = app
            .instantiate_contract(
                dao_tunnel_id,
                owner.clone(),
                &DTunnelInstanstiateMsg {
                    govec_minter: govec_addr.to_string(),
                    init_remote_tunnels: None,
                },
                &[],
                "dao-tunnel",            // label: human readible name for contract
                Some(owner.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        app.execute_contract(
            owner.clone(),
            govec_addr.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::Factory(factory_addr.to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        app.execute_contract(
            owner.clone(),
            govec_addr.clone(),
            &GovecExecuteMsg::UpdateConfigAddr {
                new_addr: vectis_wallet::UpdateAddrReq::DaoTunnel(dao_tunnel_addr.to_string()),
            },
            &[],
        )
        .map_err(|err| anyhow!(err))
        .unwrap();

        Ok(DaoChainSuite {
            app,
            owner,
            sc_factory_id,
            sc_proxy_id,
            sc_proxy_multisig_code_id,
            govec_id,
            stake_id,
            dao_tunnel_id,
            govec_addr,
            factory_addr,
            dao_tunnel_addr,
        })
    }
}
