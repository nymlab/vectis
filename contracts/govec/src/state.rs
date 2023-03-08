use cosmwasm_schema::cw_serde;
use cw20::{Logo, MarketingInfoResponse};

use cosmwasm_std::{Addr, Uint128};
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
