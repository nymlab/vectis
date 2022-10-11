use crate::error::IbcError;
pub use crate::{APP_ORDER, IBC_APP_VERSION};
use cosmwasm_std::{from_slice, to_binary, Binary, CosmosMsg, IbcOrder};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    Dispatch {
        sender: String,
        msgs: Vec<CosmosMsg>,
        callback_id: Option<String>,
    },
}

pub fn check_order(order: &IbcOrder) -> Result<(), IbcError> {
    if order != &APP_ORDER {
        Err(IbcError::InvalidChannelOrder)
    } else {
        Ok(())
    }
}

pub fn check_version(version: &str) -> Result<(), IbcError> {
    if version != IBC_APP_VERSION {
        Err(IbcError::InvalidChannelVersion(IBC_APP_VERSION))
    } else {
        Ok(())
    }
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and return an error message in this format
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StdAck {
    Result(Binary),
    Error(String),
}

impl StdAck {
    // create a serialized success message
    pub fn success(data: impl Serialize) -> Binary {
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

    pub fn unwrap_into<T: DeserializeOwned>(self) -> T {
        from_slice(&self.unwrap()).unwrap()
    }

    pub fn unwrap_err(self) -> String {
        match self {
            StdAck::Result(_) => panic!("not an error"),
            StdAck::Error(err) => err,
        }
    }
}
