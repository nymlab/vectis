use crate::{authenticator::Authenticator, Nonce};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

/// A unit capturing how a message can be authenticated
/// i.e. Controller, Guardian, other roles
/// Given the `data`, then authenticator associated with the authenticator type
/// will be able to authenticate if the message is signed by this `Entity`
#[cw_serde]
pub struct Entity {
    pub auth: Authenticator,
    /// For CosmosEOA: this is the base64 encoding of the bytes of the addr
    /// i.e. `toBase(toUtf8(addr))`
    pub data: Binary,
    pub nonce: Nonce,
}
