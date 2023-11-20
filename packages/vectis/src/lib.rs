#![allow(deprecated)]
pub mod interface;
pub mod types;
pub mod util;

//  Global settings for addr pagination
pub const MAX_LIMIT: u32 = 100;
pub const DEFAULT_LIMIT: u32 = 50;
pub const POLICY: &[u8] = "vectis-policy".as_bytes();
