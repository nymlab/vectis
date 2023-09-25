use cosmwasm_std::{coin, Binary, CanonicalAddr};
use vectis_wallet::types::{
    plugin::{PluginCodeData, PluginMetadataData},
    plugin_registry::TierDetails,
};

// Plugins for testing
// (code_id, hash, registry_id)
pub struct TestContracts {
    pub pre_tx: (u64, &'static str, u64),
    pub post_tx: (u64, &'static str, u64),
    pub exec: (u64, &'static str, u64),
    pub proxy_migrate: (u64, &'static str),
}

pub fn test_plugin_code_data(code_id: u64, code_hash: &'static str) -> PluginCodeData {
    PluginCodeData {
        latest_contract_version: VECTIS_VERSION.into(),
        new_code_id: code_id,
        new_code_hash: code_hash.into(),
    }
}

pub fn test_plugin_metadata() -> PluginMetadataData {
    PluginMetadataData {
        creator: VALID_OSMO_ADDR.into(),
        display_name: "Some-display-name".into(),
        ipfs_hash: "Some-ipfs_hash".into(),
    }
}

pub fn tier_0() -> TierDetails {
    TierDetails {
        max_plugins: 2,
        duration: None,
        fee: coin(0u128, DENOM),
    }
}

pub fn tier_1() -> TierDetails {
    TierDetails {
        max_plugins: 12,
        duration: Some(cw_utils::HOUR),
        fee: coin(TIER_1_FEE, DENOM),
    }
}

pub fn canonical_valid_osmo() -> CanonicalAddr {
    CanonicalAddr(Binary::from(
        hex::decode(HEX_CANONICAL_VALID_OSMO_ADDR)
            .unwrap()
            .as_slice(),
    ))
}

pub const VALID_OSMO_ADDR: &str = "osmo1pkf6nuq8whw5ta5537c3uqrep0yzcwkrw82n95";
pub const HEX_CANONICAL_VALID_OSMO_ADDR: &str = "0d93a9f00775dd45f6948fb11e00790bc82c3ac3";

/// Version of vectis
pub const VECTIS_VERSION: &str = "1.0.0-rc1";
/// the proxy code_hash for this vectis version
pub const PLUGIN_EXEC_HASH: &str =
    "f6ca658680716854067c020d1dfafafaaec4762054b63077489ac114dad957b7";
pub const POST_TX_HASH: &str = "8f2c5cac9ac94088345fe27973aef45ba84794225ea977a963d50dbf0edb4279";
pub const PRE_TX_HASH: &str = "3f1277bb69fd386b8a6bacc504775d3594b0e911824d25838a90c5579e3e3d90";
pub const PROXY_MIGRATION_HASH: &str =
    "5daf0ae9632f388efa7350d1e9cba7736f59dccf6bcafb08d5ef719b077359c3";

// Compiled from nymlab/cw-plus: vectis-beta-v1
pub const CW4_CODE_PATH: &str = "./artifacts/cw4_group.wasm";
pub const CW3FLEX_CODE_PATH: &str = "./artifacts/cw3_flex_multisig.wasm";

// Compiled from migration features enabled in core/proxy
pub const PROXY_MIGRATION_CODE_PATH: &str = "./artifacts/vectis_proxy_migration.wasm";
pub const PROXY_MIGRATE_VERSION: &str = "2.0.0-rc1";

// Vectis contracts
pub const FACTORY_CODE_PATH: &str = "./../../artifacts/vectis_factory.wasm";
pub const PROXY_CODE_PATH: &str = "../../artifacts/vectis_proxy.wasm";
pub const REGISTRY_CODE_PATH: &str = "./../../artifacts/vectis_plugin_registry.wasm";
pub const AUTH_CODE_PATH: &str = "./../../artifacts/vectis_webauthn_authenticator.wasm";

// Vectis test plugin contracts
pub const PRE_TX_CODE_PATH: &str = "./../../artifacts/test_vectis_pre_tx.wasm";
pub const POST_TX_CODE_PATH: &str = "./../../artifacts/test_vectis_post_tx_exec.wasm";
pub const PLUGIN_EXEC_CODE_PATH: &str = "./../../artifacts/test_vectis_plugin_exec.wasm";

pub const DENOM: &str = "uosmo";
pub const WALLET_FEE: u128 = 10u128;
pub const REGISTRY_FEE: u128 = 11u128;
pub const INIT_BALANCE: u128 = 12u128;
pub const TIER_1_FEE: u128 = 5u128;

// Indexes for test_env
pub const IDEPLOYER: usize = 0;
pub const ICOMMITTEE: usize = 1;
pub const IRELAYER: usize = 2;

// storage const
pub const IBC_CHAIN_NAME: &str = "ibc-chain-1";
pub const NON_IBC_CHAIN_NAME: &str = "non-ibc-chain-1";
pub const NON_IBC_CHAIN_CONN: &str = "some-url";
pub const NON_IBC_CHAIN_ADDR: &str = "0x123";
