use cosmwasm_schema::cw_serde;
use cosmwasm_std::{has_coins, Binary, Coin};

use crate::types::{
    authenticator::AuthenticatorType,
    error::{FactoryError, MigrationMsgError},
    plugin::PluginInstallParams,
    wallet::{Controller, RelayTransaction},
};

/// Declares that a fixed weight of Yes votes is needed to pass.
/// See `ThresholdResponse.AbsoluteCount` in the cw3 spec for details.
/// Only Fixed multisig is supported in this version
pub type ThresholdAbsoluteCount = u64;

#[cw_serde]
pub struct CreateWalletMsg {
    /// The main controller of the wallet
    pub controller: Controller,
    /// A list of keys can act as relayer for
    pub relayers: Vec<String>,
    /// The fund to send to the wallet initially when it is created
    pub proxy_initial_funds: Vec<Coin>,
    /// Vectis ID of the wallet which must be unique
    pub vid: String,
    /// Initial data to be set in the proxy
    /// Record is a kv pair  
    pub initial_data: Vec<(Binary, Binary)>,
    /// Initial plugins to be instantiated
    pub plugins: Vec<PluginInstallParams>,
}

impl CreateWalletMsg {
    pub fn ensure_has_sufficient_funds(&self, funds: Vec<Coin>) -> Result<(), FactoryError> {
        // sum up all coins required
        // required = proxy_initial_funds + each fund for instantiating plugin which
        let required_coins = if self.plugins.is_empty() {
            self.proxy_initial_funds.clone()
        } else {
            let mut required_coins = self.proxy_initial_funds.clone();
            // Adds up all the funds required to install the plugins
            for p in &self.plugins {
                for coin in &p.funds {
                    required_coins
                        .iter_mut()
                        .find(|c| c.denom == coin.denom)
                        .map(|c| c.amount.checked_add(coin.amount))
                        .transpose()
                        .map_err(|_| FactoryError::OverFlow {})?;
                }
            }
            required_coins
        };
        for coin in required_coins {
            if !has_coins(&funds, &coin) {
                return Err(FactoryError::InvalidSufficientFunds(
                    coin.amount.to_string(),
                    coin.denom,
                ));
            }
        }

        Ok(())
    }
}

#[cw_serde]
pub struct MigrateWalletMsg {
    pub addr_to_migrate: String,
    pub tx: RelayTransaction,
}

#[cw_serde]
pub struct ProxyMigrateMsg {
    pub new_code_id: u64,
}

impl ProxyMigrateMsg {
    /// Ensures code id of multisig contract is equal to current factory multisig code id,
    pub fn ensure_is_supported_proxy_code_id(
        &self,
        factory_proxy_code_id: u64,
    ) -> Result<(), MigrationMsgError> {
        if factory_proxy_code_id != self.new_code_id {
            return Err(MigrationMsgError::MismatchProxyCodeId);
        }
        Ok(())
    }
}

#[cw_serde]
pub struct AuthenticatorInstInfo {
    pub ty: AuthenticatorType,
    pub code_id: u64,
    pub inst_msg: Binary,
}

#[cw_serde]
pub struct WalletFactoryInstantiateMsg {
    /// Smart contract wallet contract code id
    pub proxy_code_id: u64,
    /// Fee for wallet creation in native token to be sent to Admin (DAO)
    pub wallet_fee: Coin,
    /// Authenticator
    pub authenticators: Option<Vec<AuthenticatorInstInfo>>,
    /// Supported Chains
    pub supported_chains: Option<Vec<(String, ChainConnection)>>,
}

#[cw_serde]
pub enum CodeIdType {
    Proxy,
}

#[cw_serde]
pub enum FeeType {
    Wallet,
}

#[cw_serde]
pub struct FeesResponse {
    pub wallet_fee: Coin,
}

#[cw_serde]
pub enum ChainConnection {
    /// IBC connection-id establised on hub chain
    IBC(String),
    /// Other non-IBC chains: frontend is expected to interpret this string
    /// For example some useful info for resolving, maybe an endpoint
    Other(String),
}
