/// Dao Chain govec contract reply
pub const GOVEC_REPLY_ID: u64 = u64::MIN;

pub mod factory_state {
    pub use crate::DEPLOYER;
    use cosmwasm_std::Coin;
    use cw_storage_plus::Item;
    /// The total number of wallets successfully created by the factory
    /// i.e. if creation fail, this is not incremented
    pub const TOTAL_CREATED: Item<u64> = Item::new("total_created");
    /// The latest supported `wallet_proxy` code id stored onchain
    pub const PROXY_CODE_ID: Item<u64> = Item::new("proxy_code_id");
    /// The latest default `multisig` code id stored onchain for the proxy
    pub const PROXY_MULTISIG_CODE_ID: Item<u64> = Item::new("proxy_multisig_code_id");
    /// Chain address prefix
    pub const ADDR_PREFIX: Item<String> = Item::new("addr_prefix");
    /// Fee for DAO when a wallet is created
    pub const WALLET_FEE: Item<Coin> = Item::new("wallet_fee");
}
