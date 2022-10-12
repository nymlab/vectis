use cosmwasm_std::{
    from_slice, to_binary, wasm_execute, Binary, CosmosMsg, IbcOrder, IbcReceiveResponse,
    StdResult, SubMsg, WasmMsg,
};

use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::IbcError;
pub use crate::{APP_ORDER, IBC_APP_VERSION, RECEIVE_DISPATCH_ID};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    UpdateChannel,
    InstantiateFactory {
        code_id: u64,
        msg: Option<CosmosMsg>
    },
    Dispatch {
        msg: CosmosMsg,
        sender: String,
        contract_addr: String,
        callback_id: Option<String>,
    },
    MintGovec {
        wallet_addr: String,
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

pub fn check_connection(host_connection: &str, remote_connection: &str) -> Result<(), IbcError> {
    if host_connection != remote_connection {
        return Err(IbcError::InvalidConnectionId(host_connection.to_string()));
    }
    Ok(())
}

pub fn check_port(host_port: &str, remote_port: &str) -> Result<(), IbcError> {
    if host_port != remote_port {
        return Err(IbcError::InvalidPortId(host_port.to_string()));
    }
    Ok(())
}

pub fn receive_dispatch(contract_addr: String, msg: CosmosMsg) -> StdResult<IbcReceiveResponse> {
    let acknowledgement = StdAck::success(&());

    let wasm_msg = wasm_execute(contract_addr, &msg, vec![])?;

    let msg = SubMsg::reply_on_success(wasm_msg, RECEIVE_DISPATCH_ID);

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(msg)
        .add_attribute("action", "vectis_tunnel_remote_receive_dispatch"))
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

/// ReceiveIbcResponseMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ReceiveIcaResponseMsg {
    /// The ID chosen by the caller in the `callback_id`
    pub id: String,
    pub msg: StdAck,
}

impl ReceiveIcaResponseMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = SimpleIcaReceiverExecuteMsg::ReceiveIcaResponse(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + JsonSchema,
    {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}

/// This is just a helper to properly serialize the above message.
/// The actual receiver should include this variant in the larger ExecuteMsg enum
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
enum SimpleIcaReceiverExecuteMsg {
    ReceiveIcaResponse(ReceiveIcaResponseMsg),
}
