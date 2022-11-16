/// Dao Chain govec contract reply
pub const GOVEC_REPLY_ID: u64 = u64::MIN;

pub mod factory_state {
    use cosmwasm_std::{CanonicalAddr, Coin};
    use cw_storage_plus::{Item, Map};
    use cw_utils::Expiration;
    /// The total number of wallets successfully created by the factory
    /// i.e. if creation fail, this is not incremented
    pub const TOTAL_CREATED: Item<u64> = Item::new("total_created");
    /// The DAO address where the fees for new wallet goes to, also the admin of the contract.
    /// On a remote chain, this is the remote-tunnel contract
    pub const DAO: Item<CanonicalAddr> = Item::new("admin");
    /// The latest supported `wallet_proxy` code id stored onchain
    pub const PROXY_CODE_ID: Item<u64> = Item::new("proxy_code_id");
    /// The latest default `multisig` code id stored onchain for the proxy
    pub const PROXY_MULTISIG_CODE_ID: Item<u64> = Item::new("proxy_multisig_code_id");
    /// All proxy wallets that have yet to claim their govec tokens
    pub const GOVEC_CLAIM_LIST: Map<Vec<u8>, Expiration> = Map::new("govec-claim-list");
    /// Chain address prefix
    pub const ADDR_PREFIX: Item<String> = Item::new("addr_prefix");
    /// Fee for DAO when a wallet is created
    pub const WALLET_FEE: Item<Coin> = Item::new("wallet_fee");
    /// Fee for DAO when claiming govec tokens.
    pub const CLAIM_FEE: Item<Coin> = Item::new("claim_fee");
}
