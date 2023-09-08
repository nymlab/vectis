/// Version of vectis
pub const VECTIS_VERSION: &str = "v1.0.0-rc1";
/// the proxy code_hash for this vectis version
pub const PROXY_CODE_HASH: &str =
    "e2c89cfcc7dca3ce329ae1687519682012c3c0c57a2ca36866ce5e6ffbe0476f";

pub const CW4_CODE_PATH: &str = "./artifacts/cw4_group.wasm";
pub const CW3FLEX_CODE_PATH: &str = "./artifacts/cw3_flex_multisig.wasm";
pub const FACTORY_CODE_PATH: &str = "./../../artifacts/vectis_factory.wasm";
pub const PROXY_CODE_PATH: &str = "../../artifacts/vectis_proxy.wasm";
pub const REGISTRY_CODE_PATH: &str = "./../../artifacts/vectis_plugin_registry.wasm";
pub const AUTH_CODE_PATH: &str = "./../../artifacts/vectis_webauthn_authenticator.wasm";

pub const DENOM: &str = "uosmo";
pub const WALLET_FEE: u128 = 10u128;
pub const REGISTRY_FEE: u128 = 10u128;
pub const INIT_BALANCE: u128 = 10u128;

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
