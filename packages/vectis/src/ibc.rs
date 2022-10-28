use cosmwasm_schema::{cw_serde, schemars, serde};
use cosmwasm_std::{
    from_slice, to_binary, Binary, CosmosMsg, IbcBasicResponse, IbcOrder, IbcPacketAckMsg,
    StdResult, WasmMsg,
};

pub use crate::{
    GovecExecuteMsg, IbcError, WalletFactoryInstantiateMsg, APP_ORDER, IBC_APP_VERSION,
    RECEIVE_DISPATCH_ID,
};
use cw20_stake::msg::ExecuteMsg as StakeExecuteMsg;
use cw_proposal_single::msg::ExecuteMsg as ProposalExecuteMsg;

#[cw_serde]
pub struct PacketMsg {
    pub sender: String,
    pub job_id: Option<String>,
    // This can only be DaoTunnelPacketMsg or RemoteTunnelPacketMsg
    pub msg: Binary,
}

/// The IBC Packet Msg allowed dispatched by dao-tunnel
#[cw_serde]
pub enum DaoTunnelPacketMsg {
    UpdateChannel,
    InstantiateFactory {
        code_id: u64,
        msg: WalletFactoryInstantiateMsg,
    },
    MintGovec {
        wallet_addr: String,
    },
}

/// The IBC Packet Msg allowed dispatched by remote-tunnel
#[cw_serde]
pub enum RemoteTunnelPacketMsg {
    MintGovec { wallet_addr: String },
    GovecActions(GovecExecuteMsg),
    StakeActions(StakeExecuteMsg),
    ProposalActions(ProposalExecuteMsg),
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

pub fn acknowledge_dispatch(
    job_id: Option<String>,
    sender: String,
    ack: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let res = IbcBasicResponse::new().add_attribute("action", "acknowledge_dispatch");
    match job_id {
        Some(id) => {
            let msg: StdAck = from_slice(&ack.acknowledgement.data)?;
            // Send IBC packet ack message to another contract
            let res = res
                .add_attribute("job_id", &id)
                .add_message(ReceiveIbcResponseMsg { id: id, msg }.into_cosmos_msg(sender)?);
            Ok(res)
        }
        None => Ok(res),
    }
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

/// ReceiveIbcResponseMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
#[cw_serde]
pub struct ReceiveIbcResponseMsg {
    /// The ID chosen by the caller in the `job_id`
    pub id: String,
    pub msg: StdAck,
}

impl ReceiveIbcResponseMsg {
    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = SimpleIbcReceiverExecuteMsg::ReceiveIcaResponse(self);
        to_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>, C>(self, contract_addr: T) -> StdResult<CosmosMsg<C>>
    where
        C: Clone + std::fmt::Debug + PartialEq + schemars::JsonSchema,
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
#[cw_serde]
enum SimpleIbcReceiverExecuteMsg {
    ReceiveIcaResponse(ReceiveIbcResponseMsg),
}

/// Return the data field for each message
#[cw_serde]
pub struct DispatchResponse {
    pub results: Vec<Binary>,
}
