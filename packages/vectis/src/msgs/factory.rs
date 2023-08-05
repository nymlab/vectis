use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Coin};

use crate::{Controller, Guardians, MigrationMsgError, RelayTransaction, WalletAddr};

/// Declares that a fixed weight of Yes votes is needed to pass.
/// See `ThresholdResponse.AbsoluteCount` in the cw3 spec for details.
/// Only Fixed multisig is supported in this version
pub type ThresholdAbsoluteCount = u64;

#[cw_serde]
pub struct CreateWalletMsg {
    pub controller: Controller,
    pub guardians: Guardians,
    /// A List of keys can act as relayer for
    pub relayers: Vec<String>,
    pub proxy_initial_funds: Vec<Coin>,
    pub label: String,
}

#[cw_serde]
#[derive(Default)]
pub struct MultiSig {
    // Declares that a fixed weight of Yes votes is needed to pass.
    /// Only Fixed multisig is supported in this version
    pub threshold_absolute_count: ThresholdAbsoluteCount,
}

#[cw_serde]
pub enum CodeIdType {
    Proxy,
    Multisig,
}

#[cw_serde]
pub enum ProxyMigrationTxMsg {
    RelayTx(RelayTransaction),
    DirectMigrationMsg(Binary),
}

#[cw_serde]
pub struct ProxyMigrateMsg {
    pub new_code_id: u64,
}

impl ProxyMigrateMsg {
    /// Ensures code id of multisig contract is equal to current factory multisig code id,
    pub fn ensure_is_supported_proxy_code_id(
        &self,
        factory_proxy_code_id: u64,
    ) -> Result<(), MigrationMsgError> {
        if factory_proxy_code_id != self.new_code_id {
            return Err(MigrationMsgError::MismatchProxyCodeId);
        }
        Ok(())
    }
}

#[cw_serde]
pub struct WalletFactoryInstantiateMsg {
    /// Smart contract wallet contract code id
    pub proxy_code_id: u64,
    /// Wallet guardians multisig contract code id
    /// Currently v0.13.0 cw-plus cw3-fixed-multisig
    pub proxy_multisig_code_id: u64,
    /// Chain address prefix
    pub addr_prefix: String,
    /// Fee for wallet creation in native token to be sent to Admin (DAO)
    pub wallet_fee: Coin,
}

#[cw_serde]
pub enum WalletFactoryExecuteMsg {
    CreateWallet {
        create_wallet_msg: CreateWalletMsg,
    },
    MigrateWallet {
        wallet_address: WalletAddr,
        migration_msg: ProxyMigrationTxMsg,
    },
    UpdateCodeId {
        #[serde(rename = "type")]
        ty: CodeIdType,
        new_code_id: u64,
    },
    UpdateConfigFee {
        #[serde(rename = "type")]
        ty: FeeType,
        new_fee: Coin,
    },
    UpdateDeployer {
        addr: String,
    },
    UpdateController {
        old_controller: Addr,
        new_controller: Addr,
    },
    UpdateGuardians {
        old_guardians: Vec<Addr>,
        new_guardians: Vec<Addr>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum WalletFactoryQueryMsg {
    #[returns(u64)]
    TotalCreated {},
    #[returns(u64)]
    CodeId { ty: CodeIdType },
    /// Returns the fees required to create a wallet
    #[returns(FeesResponse)]
    Fees {},
    /// Returns the address of the Deployer which holds the admin role of this contract
    #[returns(Addr)]
    DeployerAddr {},
    /// Returns the wallet controlled by this controller
    #[returns(Vec<Addr>)]
    ControllerWallets { controller: Addr },
    /// Returns the wallet with this guardian
    #[returns(Vec<Addr>)]
    WalletsWithGuardian { guardian: Addr },
}

#[cw_serde]
pub enum FeeType {
    Wallet,
}

#[cw_serde]
pub struct FeesResponse {
    pub wallet_fee: Coin,
}
