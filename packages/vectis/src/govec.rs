use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint128};
pub use cw20::{
    AllAccountsResponse, BalanceResponse, Cw20Coin, DownloadLogoResponse, Logo,
    MarketingInfoResponse, TokenInfoResponse,
};

/// Time allowed for user to claim their Govec
pub const GOVEC_CLAIM_DURATION_DAY_MUL: u64 = 90;

#[cw_serde]
pub enum UpdateAddrReq {
    Dao(String),
    DaoTunnel(String),
    Factory(String),
    Staking(String),
    Proposal(String),
}

#[cw_serde]
pub enum GovecExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
        relayed_from: Option<String>,
    },
    /// This is implemented such that the sender MUST be a DAO proposal module
    /// The recipient will always be the sender
    /// This matches the cw_proposal_single `get_deposit_msg` required interface
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    /// Burn is a base message to destroy tokens forever
    /// Logic checks that caller only has exactly 1 vote token in their balance
    Burn { relayed_from: Option<String> },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
        relayed_from: Option<String>,
    },
    /// If authorized, creates 1 new vote token and adds to the new wallets .
    Mint { new_wallet: String },
    /// Updates the mint cap of the contract.Authorized by the DAO
    /// permission: executed by dao only
    UpdateMintCap { new_mint_cap: Option<Uint128> },
    /// Updates the staking contract address.Authorized by the DAO
    /// permission: executed by dao only
    UpdateConfigAddr { new_addr: UpdateAddrReq },
    /// If authorized, updates marketing metadata.
    /// Setting None/null for any of these will leave it unchanged.
    /// Setting Some("") will clear this field on the contract storage
    /// permission: executed by dao only
    UpdateMarketing {
        /// A URL pointing to the project behind this token.
        project: Option<String>,
        /// A longer description of the token and it's utility. Designed for tooltips or such
        description: Option<String>,
        /// The address (if any) who can update this data structure
        marketing: Option<String>,
    },
    /// If set as the "marketing" role on the contract, upload a new URL, SVG, or PNG for the token
    UploadLogo(Logo),
}

#[cw_serde]
pub struct MintResponse {
    pub minters: Option<Vec<String>>,
    pub cap: Option<Uint128>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum GovecQueryMsg {
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
