use anyhow::{anyhow, Result};
use assert_matches::assert_matches;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Empty, QueryRequest, StdError,
    WasmMsg, WasmQuery,
};
use cw3::Vote;
use cw3_fixed_multisig::contract::{
    execute as fixed_multisig_execute, instantiate as fixed_multisig_instantiate,
    query as fixed_multisig_query,
};
use cw3_fixed_multisig::msg::ExecuteMsg as MultisigExecuteMsg;
use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
use derivative::Derivative;
use sc_wallet::{
    CreateWalletMsg, Guardians, MigrateMsg, MultiSig, ProxyMigrationMsg, RelayTransaction,
    ThresholdAbsoluteCount, WalletAddr, WalletInfo,
};
use secp256k1::{bitcoin_hashes::sha256, Message, PublicKey, Secp256k1, SecretKey};
use serde::de::DeserializeOwned;
use wallet_factory::{
    contract::{
        execute as factory_execute, instantiate as factory_instantiate, query as factory_query,
        reply as factory_reply,
    },
    msg::{
        ExecuteMsg as FactoryExecuteMsg, InstantiateMsg, QueryMsg as FactoryQueryMsg,
        WalletListResponse,
    },
};
use wallet_proxy::{
    contract::{
        execute as proxy_execute, instantiate as proxy_instantiate, migrate as proxy_migrate,
        query as proxy_query, reply as proxy_reply,
    },
    msg::ExecuteMsg as ProxyExecuteMsg,
    msg::QueryMsg as ProxyQueryMsg,
};

const USER_PRIV: &[u8; 32] = &[
    239, 236, 251, 133, 8, 71, 212, 110, 21, 151, 36, 77, 3, 214, 164, 195, 116, 229, 169, 120,
    185, 197, 114, 54, 55, 35, 162, 124, 200, 2, 59, 26,
];

const MULTISIG_THRESHOLD: ThresholdAbsoluteCount = 2;
const GUARD1: &str = "guardian1";
const GUARD2: &str = "guardian2";

fn contract_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(factory_execute, factory_instantiate, factory_query)
        .with_reply(factory_reply);
    Box::new(contract)
}

fn contract_proxy() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(proxy_execute, proxy_instantiate, proxy_query)
        .with_migrate(proxy_migrate)
        .with_reply(proxy_reply);
    Box::new(contract)
}

fn contract_multisig() -> Box<dyn Contract<Empty>> {
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
    sc_factory_id: u64,
    // ID of stored code for proxy
    sc_proxy_id: u64,
    // ID of stored code for proxy multisig
    sc_proxy_multisig_code_id: u64,
}

impl Suite {
    pub fn init() -> Result<Suite> {
        let genesis_funds = vec![coin(100000, "ucosm")];
        let owner = Addr::unchecked("owner");
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &owner, genesis_funds)
                .unwrap();
        });

        let sc_factory_id = app.store_code(contract_factory());
        let sc_proxy_id = app.store_code(contract_proxy());
        let sc_proxy_multisig_code_id = app.store_code(contract_multisig());

        Ok(Suite {
            app,
            owner,
            sc_factory_id,
            sc_proxy_id,
            sc_proxy_multisig_code_id,
        })
    }

    pub fn instantiate_factory(
        &mut self,
        proxy_code_id: u64,
        proxy_multisig_code_id: u64,
        init_funds: Vec<Coin>,
    ) -> Addr {
        self.app
            .instantiate_contract(
                self.sc_factory_id,                  // code_id: u64,
                Addr::unchecked(self.owner.clone()), // sender: Addr,
                &InstantiateMsg {
                    proxy_code_id,
                    proxy_multisig_code_id,
                }, // InstantiateMsg
                &init_funds,
                "wallet-factory", // label: human readible name for contract
                None,             // admin: Option<String>, will need this for upgrading
            )
            .unwrap()
    }

    pub fn create_new_proxy(
        &mut self,
        factory: Addr,
        initial_fund: Vec<Coin>,
        guardians_multisig: Option<MultiSig>,
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
            proxy_initial_funds: initial_fund,
        };

        let execute = FactoryExecuteMsg::CreateWallet { create_wallet_msg };

        self.app
            .execute_contract(
                self.owner.clone(), //sender: Addr,
                factory,            //contract_addr: Addr,
                &execute,           //msg: &T,
                &[],                //send_funds: &[Coin],
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
                &FactoryExecuteMsg::UpdateProxyCodeId { new_code_id },
                &[],
            )
            .map_err(|err| anyhow!(err))
    }

    pub fn query_wallet_addresses(
        &self,
        contract_addr: &Addr,
    ) -> Result<WalletListResponse, StdError> {
        let r: WalletListResponse =
            self.app
                .wrap()
                .query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&FactoryQueryMsg::Wallets {}).unwrap(),
                }))?;
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

    pub fn query_balance(&self, addr: &Addr, denom: String) -> Result<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), denom)?)
    }
}
#[test]
fn create_new_proxy() {
    let mut suite = Suite::init().unwrap();

    let genesis_fund: Coin = coin(1000, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![genesis_fund.clone()],
    );

    let init_wallet_fund: Coin = coin(100, "ucosm");
    let rsp = suite.create_new_proxy(factory.clone(), vec![init_wallet_fund.clone()], None);
    assert_matches!(rsp, Ok(_));

    let mut r = suite.query_wallet_addresses(&factory).unwrap();
    assert_matches!(r.wallets.len(), 1);
    let wallet_addr = r.wallets.pop().unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    let factory_fund = suite.query_balance(&factory, "ucosm".into()).unwrap();
    let wallet_fund = suite.query_balance(&wallet_addr, "ucosm".into()).unwrap();

    assert_eq!(
        genesis_fund.amount - factory_fund.amount,
        wallet_fund.amount
    );
    assert_eq!(w.code_id, suite.sc_proxy_id);
    assert!(w.guardians.contains(&Addr::unchecked(GUARD1)));
    assert!(!w.is_frozen);
}

#[test]
fn user_can_execute_message() {
    let mut suite = Suite::init().unwrap();
    let genesis_fund: Coin = coin(1000, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![genesis_fund.clone()],
    );
    let init_wallet_fund: Coin = coin(100, "ucosm");
    let create_proxy_rsp =
        suite.create_new_proxy(factory.clone(), vec![init_wallet_fund.clone()], None);
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;
    let send_amount: Coin = coin(10, "ucosm");

    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: factory.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        user,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute { msgs: vec![msg] },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let wallet_fund = suite
        .query_balance(&wallet_address, "ucosm".into())
        .unwrap();

    assert_eq!(
        init_wallet_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn create_new_proxy_with_multisig_guardians() {
    let mut suite = Suite::init().unwrap();

    let genesis_fund: Coin = coin(1000, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![genesis_fund.clone()],
    );

    let init_wallet_fund: Coin = coin(100, "ucosm");

    let multisig = MultiSig {
        threshold_absolute_count: MULTISIG_THRESHOLD,
        multisig_initial_funds: vec![],
    };

    let rsp = suite.create_new_proxy(
        factory.clone(),
        vec![init_wallet_fund.clone()],
        Some(multisig),
    );
    assert_matches!(rsp, Ok(_));

    let mut r = suite.query_wallet_addresses(&factory).unwrap();
    assert_matches!(r.wallets.len(), 1);
    let wallet_addr = r.wallets.pop().unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    // Test wallet freezing, when multisig scenario is enabled
    assert!(!w.is_frozen);

    let multisig_contract_addr = w.multisig_address.unwrap();

    let execute_revert_freeze_status_msg = WasmMsg::Execute {
        contract_addr: wallet_addr.to_string(),
        msg: to_binary(&ProxyExecuteMsg::<Empty>::RevertFreezeStatus {}).unwrap(),
        funds: vec![],
    };

    let multisig_propose_msg = MultisigExecuteMsg::Propose {
        title: "Revert freeze status".to_string(),
        description: "Need to revert freeze status".to_string(),
        msgs: vec![execute_revert_freeze_status_msg.into()],
        latest: None,
    };

    // propose wallet revert freeze status
    // first proposer has considered cast a ballot
    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD1),
            multisig_contract_addr.clone(),
            &multisig_propose_msg,
            &[],
        )
        .unwrap();

    // vote msg
    let vote_msg = MultisigExecuteMsg::Vote {
        proposal_id: 1,
        vote: Vote::Yes,
    };

    // second vote
    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD2),
            multisig_contract_addr.clone(),
            &vote_msg,
            &[],
        )
        .unwrap();

    // execute proposal
    let execute_proposal_msg = MultisigExecuteMsg::Execute { proposal_id: 1 };

    suite
        .app
        .execute_contract(
            Addr::unchecked(GUARD1),
            multisig_contract_addr,
            &execute_proposal_msg,
            &[],
        )
        .unwrap();

    let w: WalletInfo = suite.query_wallet_info(&wallet_addr).unwrap();

    // Ensure freezing msg passed
    assert!(w.is_frozen);
}
// Migration related tests
#[test]
fn user_can_migrate_with_direct_message() {
    let mut suite = Suite::init().unwrap();
    let init_wallet_fund: Coin = coin(100, "ucosm");
    let factory = suite.instantiate_factory(
        suite.sc_proxy_id,
        suite.sc_proxy_multisig_code_id,
        vec![init_wallet_fund],
    );
    let init_proxy_fund: Coin = coin(90, "ucosm");
    let create_proxy_rsp =
        suite.create_new_proxy(factory.clone(), vec![init_proxy_fund.clone()], None);
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let user = w.user_addr;
    let old_code_id = w.code_id;
    assert_eq!(old_code_id, suite.sc_proxy_id);

    let new_code_id = suite.app.store_code(contract_proxy());
    let r = suite.update_proxy_code_id(new_code_id, factory.clone());
    assert!(r.is_ok());

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        proxy_migration_msg: ProxyMigrationMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id,
                msg: to_binary(&MigrateMsg { new_code_id }).unwrap(),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_resp =
        suite
            .app
            .execute_contract(user.clone(), factory.clone(), &migrate_wallet_msg, &[]);

    assert!(execute_msg_resp.is_ok());
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, new_code_id);
    assert_ne!(new_code_id, old_code_id);

    // user can execute message after migration
    let send_amount: Coin = coin(10, "ucosm");
    let msg = CosmosMsg::<()>::Bank(BankMsg::Send {
        to_address: factory.to_string(),
        amount: vec![send_amount.clone()],
    });

    let execute_msg_resp = suite.app.execute_contract(
        user,
        wallet_address.clone(),
        &ProxyExecuteMsg::Execute { msgs: vec![msg] },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let wallet_fund = suite
        .query_balance(&wallet_address, "ucosm".into())
        .unwrap();

    assert_eq!(
        init_proxy_fund.amount - send_amount.amount,
        wallet_fund.amount
    );
}

#[test]
fn relayer_can_migrate_with_user_signature() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);
    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let old_code_id = w.code_id;
    let relayer = w.relayers.pop().unwrap();
    assert_eq!(old_code_id, suite.sc_proxy_id);

    let new_code_id = suite.app.store_code(contract_proxy());
    let r = suite.update_proxy_code_id(new_code_id, factory.clone());
    assert!(r.is_ok());

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id,
        msg: to_binary(&MigrateMsg { new_code_id }).unwrap(),
    });

    let relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);
    println!("{:?}", relay_transaction.signature);

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        factory.clone(),
        &FactoryExecuteMsg::MigrateWallet {
            wallet_address: WalletAddr::Addr(wallet_address.clone()),
            proxy_migration_msg: ProxyMigrationMsg::RelayTx(relay_transaction),
        },
        &[],
    );
    assert!(execute_msg_resp.is_ok());

    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, new_code_id);
    assert_ne!(new_code_id, old_code_id);
}

#[test]
fn user_cannot_migrate_others_wallet() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);

    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let code_id = w.code_id;

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        proxy_migration_msg: ProxyMigrationMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id: code_id,
                msg: to_binary(&MigrateMsg {
                    new_code_id: code_id,
                })
                .unwrap(),
            }))
            .unwrap(),
        ),
    };

    let not_user = Addr::unchecked("not_user");

    let execute_msg_resp = suite
        .app
        .execute_contract(not_user, factory, &migrate_wallet_msg, &[]);

    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("Unauthorized")
    );
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, code_id);
}

#[test]
fn user_cannot_migrate_with_mismatched_code_id() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let code_id = w.code_id;

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        proxy_migration_msg: ProxyMigrationMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::Migrate {
                contract_addr: wallet_address.to_string(),
                new_code_id: code_id + 122,
                msg: to_binary(&MigrateMsg {
                    new_code_id: code_id,
                })
                .unwrap(),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_resp =
        suite
            .app
            .execute_contract(w.user_addr, factory, &migrate_wallet_msg, &[]);

    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("InvalidMigrationMsg: MismatchCodeId")
    );
    let new_w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    assert_eq!(new_w.code_id, code_id);
}

#[test]
fn user_cannot_migrate_with_invalid_wasm_msg() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);
    assert!(create_proxy_rsp.is_ok());

    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();

    // User migrates their wallet to the new code id
    let migrate_wallet_msg = FactoryExecuteMsg::MigrateWallet {
        wallet_address: WalletAddr::Addr(wallet_address.clone()),
        proxy_migration_msg: ProxyMigrationMsg::DirectMigrationMsg(
            to_binary(&CosmosMsg::<()>::Wasm(WasmMsg::ClearAdmin {
                contract_addr: String::from("randomaddr"),
            }))
            .unwrap(),
        ),
    };

    let execute_msg_resp =
        suite
            .app
            .execute_contract(w.user_addr, factory, &migrate_wallet_msg, &[]);

    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("InvalidMigrationMsg: InvalidWasmMsg")
    );
}

#[test]
fn relayer_cannot_migrate_others_wallet() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);

    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&MigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce + 123);

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        factory.clone(),
        &FactoryExecuteMsg::MigrateWallet {
            wallet_address: WalletAddr::Addr(wallet_address.clone()),
            proxy_migration_msg: ProxyMigrationMsg::RelayTx(relay_transaction),
        },
        &[],
    );
    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("InvalidRelayMigrationTx: MismatchNonce")
    );
}

#[test]
fn relayer_cannot_migrate_with_mismatch_user_addr() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);
    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&MigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let mut relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    // invalid user_pubkey
    relay_transaction.user_pubkey = Binary([0; 33].to_vec());

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        factory.clone(),
        &FactoryExecuteMsg::MigrateWallet {
            wallet_address: WalletAddr::Addr(wallet_address.clone()),
            proxy_migration_msg: ProxyMigrationMsg::RelayTx(relay_transaction),
        },
        &[],
    );
    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("InvalidRelayMigrationTx: MismatchUserAddr")
    );
}

#[test]
fn relayer_cannot_migrate_with_invalid_signature() {
    let mut suite = Suite::init().unwrap();
    let factory =
        suite.instantiate_factory(suite.sc_proxy_id, suite.sc_proxy_multisig_code_id, vec![]);
    let create_proxy_rsp = suite.create_new_proxy(factory.clone(), vec![], None);
    assert!(create_proxy_rsp.is_ok());
    let wallet_address = suite
        .query_wallet_addresses(&factory)
        .unwrap()
        .wallets
        .pop()
        .unwrap();
    let mut w: WalletInfo = suite.query_wallet_info(&wallet_address).unwrap();
    let relayer = w.relayers.pop().unwrap();

    let migrate_msg = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: wallet_address.to_string(),
        new_code_id: 0,
        msg: to_binary(&MigrateMsg { new_code_id: 0 }).unwrap(),
    });

    let mut relay_transaction = suite.create_relay_transaction(USER_PRIV, migrate_msg, w.nonce);

    // invalid signature
    relay_transaction.signature = Binary(
        [
            1, 210, 8, 128, 147, 77, 89, 146, 29, 147, 127, 232, 221, 182, 94, 13, 73, 114, 228,
            48, 12, 21, 115, 63, 52, 224, 231, 92, 110, 8, 80, 30, 17, 93, 50, 211, 114, 25, 194,
            139, 64, 172, 4, 135, 99, 63, 178, 84, 1, 138, 204, 203, 229, 83, 249, 167, 42, 106,
            33, 109, 1, 1, 1, 1,
        ]
        .to_vec(),
    );

    let execute_msg_resp = suite.app.execute_contract(
        relayer,
        factory.clone(),
        &FactoryExecuteMsg::MigrateWallet {
            wallet_address: WalletAddr::Addr(wallet_address.clone()),
            proxy_migration_msg: ProxyMigrationMsg::RelayTx(relay_transaction),
        },
        &[],
    );
    assert_eq!(
        execute_msg_resp.unwrap_err().to_string(),
        String::from("InvalidRelayMigrationTx: SignatureVerificationError")
    );
}
