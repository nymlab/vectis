use cosmwasm_schema::{cw_serde, serde};
use cosmwasm_std::{from_slice, to_binary, Binary, CanonicalAddr, IbcOrder};
use std::convert::TryFrom;

use crate::IbcError;

pub const IBC_APP_VERSION: &str = "vectis-v1";
pub const IBC_APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[cw_serde]
pub enum VectisDaoActionIds {
    GovecMint = 10,
    GovecSend,
    GovecTransfer,
    GovecExit,
    StakeUnstake,
    StakeClaim,
    ProposalPropose,
    ProposalVote,
    ProposalExecute,
    ProposalClose,
    FactoryInstantiated,
}

impl TryFrom<u64> for VectisDaoActionIds {
    type Error = IbcError;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            10 => Ok(Self::GovecMint),
            11 => Ok(Self::GovecSend),
            12 => Ok(Self::GovecTransfer),
            13 => Ok(Self::GovecExit),
            14 => Ok(Self::StakeUnstake),
            15 => Ok(Self::StakeClaim),
            16 => Ok(Self::ProposalPropose),
            17 => Ok(Self::ProposalVote),
            18 => Ok(Self::ProposalExecute),
            19 => Ok(Self::ProposalClose),
            20 => Ok(Self::FactoryInstantiated),
            _ => Err(IbcError::InvalidDaoActionId {}),
        }
    }
}

#[cw_serde]
pub struct DaoConfig {
    /// DAO addr on dao chain
    pub addr: String,
    /// The src.port_id from the connection
    /// This is bounded to the contract address on the remote chain
    /// `wasm.<contract_address>`, i.e. the dao-tunnel contract address
    pub dao_tunnel_port_id: String,
    /// The local connection_id that is bounded to the remote chain light client
    /// This can be queried by the using the `IbcPacket` when receiving the ibc message
    /// IbcPacket.dest.channel_id and IbcPacket.dest.port_id
    pub connection_id: String,
    /// The channel_id to be used to call to the dao-tunnel contract on dao-chain
    /// This can be updated by the dao-tunnel forwarding message from the DAO
    pub dao_tunnel_channel: Option<String>,
}

#[cw_serde]
pub struct ChainConfig {
    /// The Factory that has the remote features on the local chain
    pub remote_factory: Option<CanonicalAddr>,
    /// Denom of the current chain
    pub denom: String,
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and return an error message in this format
#[cw_serde]
pub enum StdAck {
    Result(Binary),
    Error(String),
}

impl StdAck {
    // create a serialized success message
    pub fn success(data: impl serde::Serialize) -> Binary {
        let res = to_binary(&data).unwrap();
        StdAck::Result(res).ack()
    }

    // create a serialized error message
    pub fn fail(err: String) -> Binary {
        StdAck::Error(err).ack()
    }

    pub fn ack(&self) -> Binary {
        to_binary(self).unwrap()
    }

    pub fn unwrap(self) -> Binary {
        match self {
            StdAck::Result(data) => data,
            StdAck::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_into<T: serde::de::DeserializeOwned>(self) -> T {
        from_slice(&self.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> String {
        match self {
            StdAck::Result(_) => panic!("not an error"),
            StdAck::Error(err) => err,
        }
    }
}

#[cw_serde]
pub struct Receiver {
    pub connection_id: String,
    pub addr: String,
}

/// Returned when IBC_TRANSFER_MODULES are queried
#[cw_serde]
pub struct IbcTransferChannels {
    /// (connection_id, channel_id)
    /// The channel_id are for channel already established
    pub endpoints: Vec<(String, String)>,
}
