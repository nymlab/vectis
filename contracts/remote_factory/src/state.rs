use cw_storage_plus::Map;

pub use vectis_wallet::{factory_state::*, WalletInfo};

/// A temp storage for remote chains only, to ensure actually minted on dao-chain
/// `Expiration` is updated so that users can try to mint again if failed
pub const PENDING_CLAIM_LIST: Map<Vec<u8>, ()> = Map::new("govec-pending-claim-list");
