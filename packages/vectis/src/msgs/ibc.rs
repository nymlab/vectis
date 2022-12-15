use cosmwasm_schema::{cw_serde, schemars};
use cosmwasm_std::{to_binary, Binary, CosmosMsg, StdResult, WasmMsg};

pub use cw20_stake::msg::ExecuteMsg as StakeExecuteMsg;
pub use cw_proposal_single::msg::ExecuteMsg as ProposalExecuteMsg;

pub use crate::{
    ChainConfig, DaoConfig, GovecExecuteMsg, IbcError, Receiver, StdAck, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg,
};

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
        new_denom: String,
        new_remote_factory: Option<String>,
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
pub enum RemoteTunnelExecuteMsg {
    /// Executed by proxy wallets for Dao actions
    DaoActions { msg: RemoteTunnelPacketMsg },
    /// Transfer native tokens to another chain
    /// Fund amount is forward from the MessageInfo.funds
    IbcTransfer { receiver: Receiver },
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
