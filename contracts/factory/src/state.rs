use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};
pub use sc_wallet::WalletInfo;

/// The total number of wallets successfully created by the factory
/// i.e. if creation fail, this is not incremented
pub const TOTAL_CREATED: Item<u64> = Item::new("total_created");
/// The admin who can create new wallet
pub const ADMIN: Item<CanonicalAddr> = Item::new("admin");
/// The latest supported `wallet_proxy` code id stored onchain
pub const PROXY_CODE_ID: Item<u64> = Item::new("proxy_code_id");
/// The latest supported `multisig` code id stored onchain for the proxy
pub const PROXY_MULTISIG_CODE_ID: Item<u64> = Item::new("proxy_multisig_code_id");
/// All created wallets by CanonicalAddr
pub const WALLETS: Map<&[u8], ()> = Map::new("wallets");
