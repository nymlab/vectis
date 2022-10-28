use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Uint128};
use cw20::Logo;

/// Time allowed for user to claim their Govec
pub const GOVEC_CLAIM_DURATION_DAY_MUL: u64 = 90;

#[cw_serde]
pub enum UpdateAddrReq {
    Dao(String),
    DaoTunnel(String),
    Factory(String),
    Staking(String),
}

#[cw_serde]
pub enum GovecExecuteMsg {
    /// Transfer is a base message to move tokens to another account without triggering actions
    Transfer {
        recipient: String,
        amount: Uint128,
        remote_from: Option<String>,
    },
    /// Burn is a base message to destroy tokens forever
    /// Logic checks that caller only has exactly 1 vote token in their balance
    Burn { remote_from: Option<String> },
    /// Send is a base message to transfer tokens to a contract and trigger an action
    /// on the receiving contract.
    Send {
        contract: String,
        amount: Uint128,
        msg: Binary,
        remote_from: Option<String>,
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
