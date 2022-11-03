pub use crate::error::{IbcError, MigrationMsgError, RelayTxError};
pub use crate::factory::{
    CodeIdType, CreateWalletMsg, MultiSig, ProxyMigrateMsg, ProxyMigrationTxMsg,
    ThresholdAbsoluteCount, UnclaimedWalletList, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg, WalletFactoryQueryMsg,
};
pub use crate::govec::{
    GovecExecuteMsg, GovecQueryMsg, MintResponse, UpdateAddrReq, GOVEC_CLAIM_DURATION_DAY_MUL,
};
pub use crate::guardians::*;
pub use crate::ibc::{
    check_order, check_version, ChainConfig, DaoConfig, DaoTunnelPacketMsg, PacketMsg,
    ProposalExecuteMsg, ReceiveIbcResponseMsg, RemoteTunnelPacketMsg, StakeExecuteMsg, StdAck,
    VectisDaoActionIds, APP_ORDER, IBC_APP_VERSION, PACKET_LIFETIME,
};
pub use crate::pubkey::pub_key_to_address;
pub use crate::signature::query_verify_cosmos;
pub use crate::wallet::{Nonce, RelayTransaction, WalletAddr, WalletInfo};
mod error;
mod factory;
mod govec;
mod guardians;
mod ibc;
mod pubkey;
mod signature;
mod wallet;

// settings for pagination
pub const MAX_LIMIT: u32 = 100;
pub const DEFAULT_LIMIT: u32 = 50;
