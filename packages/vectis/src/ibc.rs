use cosmwasm_schema::{cw_serde, schemars, serde};
use cosmwasm_std::{
    from_slice, to_binary, Binary, CanonicalAddr, CosmosMsg, IbcOrder, StdResult, WasmMsg,
};
use std::convert::TryFrom;

pub use crate::{GovecExecuteMsg, IbcError, WalletFactoryExecuteMsg, WalletFactoryInstantiateMsg};
pub use cw20_stake::msg::ExecuteMsg as StakeExecuteMsg;
pub use cw_proposal_single::msg::ExecuteMsg as ProposalExecuteMsg;
pub const IBC_APP_VERSION: &str = "vectis-v1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[cw_serde]
pub enum VectisDaoActionIds {
    GovecMint = 10,
    GovecSend,
    GovecTransfer,
    GovecBurn,
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
            13 => Ok(Self::GovecBurn),
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
pub struct PacketMsg {
    pub sender: String,
    pub job_id: u64,
    // This can only be DaoTunnelPacketMsg or RemoteTunnelPacketMsg
    pub msg: Binary,
}

/// The IBC Packet Msg allowed dispatched by dao-tunnel
#[cw_serde]
pub enum DaoTunnelPacketMsg {
    UpdateDaoConfig {
        new_config: DaoConfig,
    },
    UpdateChainConfig {
        new_config: ChainConfig,
    },
    InstantiateFactory {
        code_id: u64,
        msg: WalletFactoryInstantiateMsg,
    },
    UpdateIbcTransferRecieverChannel {
        connection_id: String,
        // Some(new-channel-id)
        // None: if new endpoint, add to `IBC_TRANSFER_MODULES`
        // None: if already exists, delete it
        channel: Option<String>,
    },
    /// Other actions that are dispatched from dao-tunnel
    /// these do not affect the state of the remote-tunnel contract
    /// i.e. staking / unstaking native tokens
    DispatchActions {
        msgs: Vec<CosmosMsg>,
    },
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

#[cw_serde]
pub enum RemoteTunnelExecuteMsg {
    /// Executed by proxy wallets for Dao actions
    DaoActions { msg: RemoteTunnelPacketMsg },
    /// Transfer native tokens to another chain
    /// Fund amount is forward from the MessageInfo.funds
    IbcTransfer { receiver: Receiver },
}

#[cw_serde]
pub struct Receiver {
    pub connection_id: String,
    pub addr: String,
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
