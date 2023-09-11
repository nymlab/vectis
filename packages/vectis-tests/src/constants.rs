use cosmwasm_std::{coin, Binary, CanonicalAddr};
use vectis_wallet::types::{
    plugin::{PluginCodeData, PluginMetadataData},
    plugin_registry::TierDetails,
};

// Plugins for testing
// (code_id, hash, registry_id)
pub struct TestPlugins {
    pub pre_tx: (u64, &'static str, u64),
    pub post_tx: (u64, &'static str, u64),
    pub exec: (u64, &'static str, u64),
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
pub const VECTIS_VERSION: &str = "v1.0.0-rc1";
/// the proxy code_hash for this vectis version
pub const PROXY_CODE_HASH: &str =
    "906b8eecb40101c591020da36ded0e6c974136e0be83819f17e98b40d49e4be8";
pub const PLUGIN_EXEC_HASH: &str =
    "9329a0ed36848387d5c4ca20b62a87bb526d656c69cbfae18cab1d863aa39637";
pub const POST_TX_HASH: &str = "616a7e39b87ceadb959084b9112f02e53d4a937d67a1a3efbde60eed2e5ac2f1";
pub const PRE_TX_HASH: &str = "90b5cfe278e15e316a611dc5a99609be671b1941c8a146770590dad550775ef6";

pub const CW4_CODE_PATH: &str = "./artifacts/cw4_group.wasm";
pub const CW3FLEX_CODE_PATH: &str = "./artifacts/cw3_flex_multisig.wasm";
pub const FACTORY_CODE_PATH: &str = "./../../artifacts/vectis_factory.wasm";
pub const PROXY_CODE_PATH: &str = "../../artifacts/vectis_proxy.wasm";
pub const REGISTRY_CODE_PATH: &str = "./../../artifacts/vectis_plugin_registry.wasm";
pub const AUTH_CODE_PATH: &str = "./../../artifacts/vectis_webauthn_authenticator.wasm";
pub const PRE_TX_CODE_PATH: &str = "./../../artifacts/test_vectis_pre_tx.wasm";
pub const POST_TX_CODE_PATH: &str = "./../../artifacts/test_vectis_post_tx_exec.wasm";
pub const PLUGIN_EXEC_CODE_PATH: &str = "./../../artifacts/test_vectis_plugin_exec.wasm";

pub const DENOM: &str = "uosmo";
pub const WALLET_FEE: u128 = 10u128;
pub const REGISTRY_FEE: u128 = 11u128;
pub const INIT_BALANCE: u128 = 12u128;
pub const DEPLOYER_INIT_BALANCE: u128 = 100000u128;
pub const TIER_1_FEE: u128 = 5u128;

// Indexes for test_env
pub const IDEPLOYER: usize = 0;
pub const ICOMMITTEE: usize = 1;
pub const IRELAYER: usize = 2;

// storage const
pub const IBC_CHAIN_NAME: &str = "ibc-chain-1";
pub const IBC_CHAIN_CONN: &str = "connection-1";
pub const NON_IBC_CHAIN_NAME: &str = "non-ibc-chain-1";
pub const NON_IBC_CHAIN_CONN: &str = "some-url";
pub const NON_IBC_CHAIN_ADDR: &str = "0x123";
