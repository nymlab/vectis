use cosmwasm_schema::cw_serde;
use cw20::{Logo, MarketingInfoResponse};

use cosmwasm_std::{Addr, CanonicalAddr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
}

pub const TOKEN_INFO: Item<TokenInfo> = Item::new("token_info");
pub const MARKETING_INFO: Item<MarketingInfoResponse> = Item::new("marketing_info");
pub const LOGO: Item<Logo> = Item::new("logo");
pub const BALANCES: Map<&Addr, Uint128> = Map::new("balance");
pub const MINT_CAP: Item<Uint128> = Item::new("mint_cap");
pub const MINT_AMOUNT: Item<Uint128> = Item::new("Mint_amount");
// updated by `UpdateConfigAddr`
pub const STAKING_ADDR: Item<CanonicalAddr> = Item::new("staking_addr");
pub const DAO_ADDR: Item<CanonicalAddr> = Item::new("DAO_addr");
pub const DAO_TUNNEL: Item<CanonicalAddr> = Item::new("DAO_tunnel");
pub const FACTORY: Item<CanonicalAddr> = Item::new("Factory");

// We update this with the DAO, we can query the DAO but this might run out of gas,
// see https://github.com/nymlab/dao-contracts/blob/v1.0.0-vectis/contracts/cw-core/src/contract.rs#L517
pub const PRE_PROP_APPROVAL: Map<&Addr, ()> = Map::new("proposal_modules");
