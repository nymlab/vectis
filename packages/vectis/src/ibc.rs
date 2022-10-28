use cosmwasm_schema::{cw_serde, schemars, serde};
use cosmwasm_std::{from_slice, to_binary, Binary, CosmosMsg, IbcOrder, StdResult, WasmMsg};
use std::convert::TryFrom;

pub use crate::{GovecExecuteMsg, IbcError, WalletFactoryExecuteMsg, WalletFactoryInstantiateMsg};
pub use cw20_stake::msg::ExecuteMsg as StakeExecuteMsg;
pub use cw_proposal_single::msg::ExecuteMsg as ProposalExecuteMsg;
pub const IBC_APP_VERSION: &str = "vectis-v1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[cw_serde]
pub enum VectisDaoActionIds {
    GovecSend = 11,
    GovecTransfer,
    GovecBurn,
    StakeUnstake,
    StakeClaim,
    ProposalPropose,
    ProposalVote,
    ProposalExecute,
}

impl TryFrom<u64> for VectisDaoActionIds {
    type Error = IbcError;
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            11 => Ok(Self::GovecSend),
            12 => Ok(Self::GovecTransfer),
            13 => Ok(Self::GovecBurn),
            14 => Ok(Self::StakeUnstake),
            15 => Ok(Self::StakeClaim),
            16 => Ok(Self::ProposalPropose),
            17 => Ok(Self::ProposalVote),
            18 => Ok(Self::ProposalExecute),
            _ => Err(IbcError::InvalidDaoActionId {}),
        }
    }
}

#[cw_serde]
pub struct PacketMsg {
    pub sender: String,
    pub job_id: u64,
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
}

/// The IBC Packet Msg allowed dispatched by remote-tunnel
#[cw_serde]
pub enum RemoteTunnelPacketMsg {
    /// A special case where the Factory is the only one who can call this
    MintGovec {
        wallet_addr: String,
    },
    GovecActions(GovecExecuteMsg),
    StakeActions(StakeExecuteMsg),
    ProposalActions {
        prop_module_addr: String,
        msg: ProposalExecuteMsg,
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
