use cw_storage_plus::Map;
pub use vectis_wallet::{factory_state::*, WalletInfo};

/// To query the pre-proposal and dao tunnel address on the DAO core
pub const ITEMS: Map<String, String> = Map::new("items");
