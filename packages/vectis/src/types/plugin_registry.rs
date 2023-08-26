use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw_utils::{Duration, Expiration};

#[cw_serde]
pub struct TierDetails {
    pub max_plugins: u16,
    pub duration: Option<Duration>,
    pub fee: Coin,
}

#[cw_serde]
pub enum SubscriptionTier {
    Free = 0,
    L1 = 1,
    Other = 2,
}

#[cw_serde]
pub struct Subscriber {
    /// The tier
    pub tier: SubscriptionTier,
    /// Expiration of the subscription
    pub expiration: Expiration,
    /// Registry plugin ids installed
    pub plugin_installed: Vec<u64>,
}

#[cw_serde]
pub struct RegistryConfigResponse {
    pub registry_fee: Coin,
    pub deployer_addr: String,
    pub subscription_tiers: Vec<(u8, TierDetails)>,
}
