use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Uint128};
pub use cw20::{
    AllAccountsResponse, BalanceResponse, Cw20Coin, DownloadLogoResponse, MarketingInfoResponse,
    TokenInfoResponse,
};
pub use vectis_wallet::{GovecExecuteMsg as ExecuteMsg, GovecQueryMsg as QueryMsg};

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub initial_balances: Vec<Cw20Coin>,
    pub staking_addr: Option<String>,
    pub mint_cap: Option<Uint128>,
    pub factory: Option<String>,
    pub dao_tunnel: Option<String>,
    pub marketing: Option<MarketingInfoResponse>,
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        // Check name, symbol, decimals
        if !is_valid_name(&self.name) {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !is_valid_symbol(&self.symbol) {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}",
            ));
        }
        Ok(())
    }
}

fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 50 {
        return false;
    }
    true
}

fn is_valid_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 12 {
        return false;
    }
    for byte in bytes.iter() {
        if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
            return false;
        }
    }
    true
}
