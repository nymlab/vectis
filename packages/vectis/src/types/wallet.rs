use crate::types::{
    authenticator::{Authenticator, AuthenticatorType},
    entity::Entity,
    error::RelayTxError,
    factory::CreateWalletMsg,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, CanonicalAddr, Coin, CosmosMsg, Deps, StdError};

/// The message sent for instantiation
/// In this case, it is the same field as what user has input.
// We are keeping this struct for now because it used to contain other default info from the
// factory, like the guardian multisig code id etc.
#[cw_serde]
pub struct ProxyCreateMsg {
    pub create_wallet_msg: CreateWalletMsg,
}

/// Sylvia creates the instantiate message in this format - same as instantiate function signature
#[cw_serde]
pub struct ProxyInstantiateMsg {
    pub msg: ProxyCreateMsg,
}

/// The main controller of the account
pub type Controller = Entity;

/// Struct for representing a CosmosEOA as an authenticator
#[cw_serde]
pub struct CosmosEOA {
    pub addr: CanonicalAddr,
}

impl CosmosEOA {
    /// Set new controller address
    pub fn set_address(&mut self, address: CanonicalAddr) {
        self.addr = address;
    }

    /// Get human addr
    pub fn to_human(&self, deps: Deps) -> Result<Addr, StdError> {
        deps.api.addr_humanize(&self.addr)
    }
}

impl Controller {
    /// Increase nonce by 1
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Ensure nonces are equal
    pub fn ensure_nonces_are_equal(&self, nonce: &Nonce) -> Result<(), RelayTxError> {
        if self.nonce.eq(nonce) {
            Ok(())
        } else {
            Err(RelayTxError::NoncesAreNotEqual {})
        }
    }

    /// Returns the AuthenicatorType
    pub fn auth_type(&self) -> &AuthenticatorType {
        &self.auth.ty()
    }

    /// Returns the Authenicator
    pub fn authenticator(&self) -> &Authenticator {
        &self.auth
    }
}

/// Controller nonce
pub type Nonce = u64;

/// Representation of the wallet address in both form used in migration
#[cw_serde]
pub struct WalletAddrs {
    /// The chain id / name of the remote chain
    pub chain_id: String,
    /// The address associated on the remote chain
    pub addr: Option<String>,
}

#[cw_serde]
pub struct WalletInfo {
    /// The controlling entity for this wallet
    pub controller: Controller,
    /// The deployer of Vectis
    pub deployer: Addr,
    /// The version of this wallet
    pub version: cw2::ContractVersion,
    /// The code-id of this wallet,
    /// MUST always be returned for frontend
    pub code_id: u64,
    /// The relayers for wallet tx
    pub relayers: Vec<Addr>,
    /// Time of creation
    pub created_at: u64,
    /// Vectis Id
    pub vid: String,
    /// Addresses on other chains
    pub addresses: Vec<WalletAddrs>,
    /// The wallet policy
    pub policy: Option<Binary>,
}

#[cw_serde]
pub struct RelayTransaction {
    /// Message to verify,
    /// Encoding depends on the authenticator,
    /// but must contain the actual Vec<CosmosMsg> to execute
    /// e.g. the structure of WebauthnRelayedTxMsg
    pub message: Binary,
    /// Serialized signature (message + nonce).
    /// If authenticator is CosmosEOA: 64 bytes - secp256k1 verification scheme
    /// See `AuthenticatorType` for more info
    pub signature: Binary,
}

/// Data to be signed by the controlling entity
#[cw_serde]
pub struct VectisRelayedTx {
    /// messages to be executed on the entity's behalf
    pub messages: Vec<CosmosMsg>,
    /// nonce of the entity for relayed tx
    pub nonce: Nonce,
    /// fee for the relaying party
    /// We skip serializing on none because this is turned into json string to be hashed / signed
    /// must align between rust and ts tests
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sponsor_fee: Option<Coin>,
}

/// The struct that RelayTransaction.message should decode to
/// for Webauthn
#[cw_serde]
pub struct WebauthnRelayedTxMsg {
    /// This is the JSON string of the `VectisRelayTx`
    /// We parse this string in the contract for the correct type
    /// This is because we need this string to ensure fields are in the
    /// same order when hashing
    pub signed_data: String,
    pub auth_data: Binary,
    pub client_data: Binary,
}

#[cw_serde]
pub struct TestMigrateMsg {
    pub name: String,
    pub version: String,
}
