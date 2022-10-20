pub use crate::error::{IbcError, MigrationMsgError, RelayTxError};
pub use crate::factory::{
    CodeIdType, CreateWalletMsg, MultiSig, ProxyMigrateMsg, ProxyMigrationTxMsg,
    ThresholdAbsoluteCount, UnclaimedWalletList, WalletFactoryExecuteMsg,
    WalletFactoryInstantiateMsg, WalletFactoryQueryMsg,
};
pub use crate::govec::GOVEC_CLAIM_DURATION_DAY_MUL;
pub use crate::guardians::*;
pub use crate::ibc::{
    acknowledge_dispatch, check_order, check_version, DispatchResponse, PacketMsg,
    ReceiveIbcResponseMsg, StdAck,
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
use cosmwasm_std::IbcOrder;

pub const IBC_APP_VERSION: &str = "vectis-v1";
pub const APP_ORDER: IbcOrder = IbcOrder::Unordered;
pub const PACKET_LIFETIME: u64 = 60 * 60;
pub const RECEIVE_DISPATCH_ID: u64 = 1234;
