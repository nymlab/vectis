use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, CanonicalAddr};
use cw_storage_plus::Item;

#[cw_serde]
pub struct Config {
    /// The src.port_id from the connection
    /// This is bounded to the contract address on the remote chain
    /// `wasm.<contract_address>`, i.e. the dao-tunnel contract address
    pub port_id: String,
    /// The local connection_id that is bounded to the remote chain light client
    /// This can be queried by the using the `IbcPacket` when receiving the ibc message
    /// IbcPacket.dest.channel_id and IbcPacket.dest.port_id
    pub connection_id: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
/// The Factory that has the remote features on the local chain
pub const FACTORY: Item<CanonicalAddr> = Item::new("factory");
/// The channel_id to be used to call to the dao-tunnel contract on dao-chain
/// This can be updated by the dao-tunnel forwarding message from the DAO
pub const CHANNEL: Item<String> = Item::new("channel");
// this stores all results from current dispatch
pub const RESULTS: Item<Vec<Binary>> = Item::new("results");
