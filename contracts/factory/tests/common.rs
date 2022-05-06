use anyhow::{anyhow, Result};
use cosmwasm_std::{
    coin, to_binary, Addr, Binary, Coin, CosmosMsg, Empty, QueryRequest, StdError, Uint128,
    WasmQuery,
};
use cw3::VoterListResponse;
use cw3_fixed_multisig::contract::{
    execute as fixed_multisig_execute, instantiate as fixed_multisig_instantiate,
    query as fixed_multisig_query,
};
use cw3_fixed_multisig::msg::QueryMsg as MultiSigQueryMsg;
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;
use secp256k1::{bitcoin_hashes::sha256, Message, PublicKey, Secp256k1, SecretKey};
use serde::de::DeserializeOwned;
use stake_cw20::contract::{
    execute as stake_execute, instantiate as stake_instantiate, query as stake_query,
};
use vectis_factory::{
    contract::{
        execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
        reply as factory_reply,
    },
    msg::{
        ExecuteMsg as FactoryExecuteMsg, InstantiateMsg, QueryMsg as FactoryQueryMsg,
        WalletListResponse,
    },
};
use vectis_govec::{
    contract::{
        execute as govec_execute, instantiate as govec_instantiate, query as govec_query,
        reply as govec_reply,
    },
    msg::{ExecuteMsg as GovecExecuteMsg, InstantiateMsg as GovecInstantiateMsg},
    state::MinterData,
};
use vectis_proxy::{
    contract::{
        execute as proxy_execute, instantiate as proxy_instantiate, migrate as proxy_migrate,
        query as proxy_query, reply as proxy_reply,
    },
    msg::QueryMsg as ProxyQueryMsg,
};
use vectis_wallet::{
    CodeIdType, CreateWalletMsg, Guardians, MultiSig, RelayTransaction, ThresholdAbsoluteCount,
    WalletQueryPrefix,
};

pub const WALLET_FEE: u128 = 10u128;
pub const USER_PRIV: &[u8; 32] = &[
    239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];
pub const NON_USER_PRIV: &[u8; 32] = &[
    239, 111, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];
pub const USER_ADDR: &str = "wasm1ky4epcqzk0mngu7twqz06qzmpgrxstxhfch6yl";
pub const MULTISIG_THRESHOLD: ThresholdAbsoluteCount = 2;
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

pub fn contract_govec() -> Box<dyn Contract<Empty>> {
    let contract =
        ContractWrapper::new(govec_execute, govec_instantiate, govec_query).with_reply(govec_reply);
    Box::new(contract)
}

pub fn contract_stake() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(stake_execute, stake_instantiate, stake_query);
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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Suite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    /// Admin Addr
    pub owner: Addr,
    /// ID of stored code for factory
    pub sc_factory_id: u64,
    // ID of stored code for proxy
    pub sc_proxy_id: u64,
    // ID of stored code for proxy multisig
    pub sc_proxy_multisig_code_id: u64,
    // ID of stored code for govec
    pub govec_id: u64,
    // ID of stored code for staking
    pub stake_id: u64,
    // govec address
    pub govec_addr: String,
}

impl Suite {
    pub fn init() -> Result<Suite> {
        let genesis_funds = vec![coin(100000, "ucosm")];
        let owner = Addr::unchecked("owner");
        let user = Addr::unchecked(USER_ADDR);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &owner, genesis_funds)
                .unwrap();
        });
        let for_user = vec![coin(50000, "ucosm")];
        app.send_tokens(owner.clone(), user, &for_user)?;

        let sc_factory_id = app.store_code(contract_factory());
        let sc_proxy_id = app.store_code(contract_proxy());
        let sc_proxy_multisig_code_id = app.store_code(contract_multisig());
        let govec_id = app.store_code(contract_govec());
        let stake_id = app.store_code(contract_stake());

        let govec_addr = app
            .instantiate_contract(
                govec_id,
                owner.clone(),
                &GovecInstantiateMsg {
                    name: String::from("govec"),
                    symbol: String::from("gov"),
                    initial_balances: vec![],
                    staking_addr: None,
                    minter: None,
                },
                &[],
                "govec",                 // label: human readible name for contract
                Some(owner.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        Ok(Suite {
            app,
            owner,
            sc_factory_id,
            sc_proxy_id,
            sc_proxy_multisig_code_id,
            govec_id,
            stake_id,
            govec_addr: govec_addr.to_string(),
        })
    }

    pub fn instantiate_factory(
        &mut self,
        proxy_code_id: u64,
        proxy_multisig_code_id: u64,
        govec_code_id: u64,
        staking_code_id: u64,
        init_funds: Vec<Coin>,
        wallet_fee: u128,
    ) -> Addr {
        let addr = self
            .app
            .instantiate_contract(
                self.sc_factory_id,
                self.owner.clone(),
                &InstantiateMsg {
                    proxy_code_id,
                    proxy_multisig_code_id,
                    govec_code_id,
                    staking_code_id,
                    addr_prefix: "wasm".to_string(),
                    wallet_fee: Coin {
                        denom: "ucosm".to_string(),
                        amount: Uint128::new(wallet_fee),
                    },
                    govec: Some(self.govec_addr.clone()),
                },
                &init_funds,
                "wallet-factory", // label: human readible name for contract
                Some(self.owner.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        let execute = GovecExecuteMsg::UpdateMintData {
            new_mint: Some(MinterData {
                minter: addr.to_string(),
                cap: Some(Uint128::new(1000)),
            }),
        };

        self.app
            .execute_contract(
                self.owner.clone(),
                Addr::unchecked(self.govec_addr.clone()),
                &execute,
                &[],
            )
            .map_err(|err| anyhow!(err))
            .unwrap();
        addr
    }

    pub fn create_new_proxy_without_guardians(
        &mut self,
        user: Addr,
        factory: Addr,
        initial_fund: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<AppResponse> {
        let r = "relayer".to_owned();
        let secp = Secp256k1::new();

        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let create_wallet_msg = CreateWalletMsg {
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            guardians: Guardians {
                addresses: vec![],
                guardians_multisig,
            },
            relayers: vec![r],
            proxy_initial_funds: initial_fund,
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };

        self.app
            .execute_contract(
                user,
                factory,
                &execute,
                &[coin(native_tokens_amount, "ucosm")],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn create_new_proxy(
        &mut self,
        payer: Addr,
        factory: Addr,
        proxy_initial_funds: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
        // This is both the initial proxy wallet initial balance
        // and the fee for wallet creation
        native_tokens_amount: u128,
    ) -> Result<AppResponse> {
        let g1 = GUARD1.to_owned();
        let g2 = GUARD2.to_owned();

        let r = "relayer".to_owned();
        let secp = Secp256k1::new();

        let secret_key = SecretKey::from_slice(USER_PRIV).expect("32 bytes, within curve order");
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        let create_wallet_msg = CreateWalletMsg {
            user_pubkey: Binary(public_key.serialize_uncompressed().to_vec()),
            guardians: Guardians {
                addresses: vec![g1, g2],
                guardians_multisig,
            },
            relayers: vec![r],
            proxy_initial_funds,
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };

        self.app
            .execute_contract(
                payer,
                factory,
                &execute,
                &[coin(native_tokens_amount, "ucosm")],
            )
            .map_err(|err| anyhow!(err))
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
                self.owner.clone(),
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
                self.owner.clone(),
                factory,
                &FactoryExecuteMsg::UpdateCodeId {
                    ty: CodeIdType::Multisig,
                    new_code_id,
                },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn query_all_wallet_addresses(
        &self,
        contract_addr: &Addr,
        start_after: Option<WalletQueryPrefix>,
        limit: Option<u32>,
    ) -> Result<WalletListResponse, StdError> {
        let r = self
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&FactoryQueryMsg::Wallets { start_after, limit }).unwrap(),
            }))
            .unwrap();
        Ok(r)
    }

    pub fn query_user_wallet_addresses(
        &self,
        contract_addr: &Addr,
        user: &str,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> Result<WalletListResponse, StdError> {
        let r = self
            .app
            .wrap()
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&FactoryQueryMsg::WalletsOf {
                    user: user.to_string(),
                    start_after,
                    limit,
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
}
