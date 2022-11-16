use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::Item;

pub use vectis_wallet::{factory_state::*, WalletInfo};

/// Governing token minting contract
pub const GOVEC_MINTER: Item<CanonicalAddr> = Item::new("govec-minter");
