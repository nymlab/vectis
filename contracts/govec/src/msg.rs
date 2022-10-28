use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, StdError, StdResult, Uint128};
pub use cw20::{
    AllAccountsResponse, BalanceResponse, Cw20Coin, DownloadLogoResponse, MarketingInfoResponse,
    TokenInfoResponse,
};
pub use vectis_wallet::GovecExecuteMsg as ExecuteMsg;

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

#[cw_serde]
pub struct MintResponse {
    pub minters: Option<Vec<String>>,
    pub cap: Option<Uint128>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the current balance of the given address, 0 if unset.
    /// Return type: BalanceResponse.
    #[returns(BalanceResponse)]
    Balance { address: String },
    /// Returns Some(balance) if address has ever been issued a token,
    /// If the current balance is 0, returns Some(0)
    /// IF address has never been issued a token, None is returned
    #[returns(Option<BalanceResponse>)]
    Joined { address: String },
    /// Returns metadata on the contract - name, decimals, supply, etc.
    /// Return type: TokenInfoResponse.
    #[returns(TokenInfoResponse)]
    TokenInfo {},
    /// Returns who can mint and the hard cap on maximum tokens after minting.
    /// Return type: MintResponse
    #[returns(MintResponse)]
    Minters {},
    /// Returns the staking contract address
    #[returns(Addr)]
    Staking {},
    /// Returns the dao contract address
    #[returns(Addr)]
    Dao {},
    /// Returns the dao tunnel contract address
    #[returns(Addr)]
    DaoTunnel {},
    /// Returns the factory contract address
    #[returns(Addr)]
    Factory {},
    /// Only with "enumerable" extension
    /// Returns all accounts that have balances. Supports pagination.
    /// Return type: AllAccountsResponse.
    #[returns(AllAccountsResponse)]
    AllAccounts {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Returns more metadata on the contract to display in the client:
    /// - description, logo, project url, etc.
    /// Return type: MarketingInfoResponse
    #[returns(MarketingInfoResponse)]
    MarketingInfo {},
    /// Downloads the embedded logo data (if stored on chain). Errors if no logo data is stored for this
    /// contract.
    /// Return type: DownloadLogoResponse.
    #[returns(DownloadLogoResponse)]
    DownloadLogo {},
}
