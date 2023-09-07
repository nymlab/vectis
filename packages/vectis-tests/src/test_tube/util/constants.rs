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
pub const MINT_AMOUNT: u128 = 2u128;
pub const WALLET_FEE: u128 = 10u128;
pub const REGISTRY_FEE: u128 = 10u128;
pub const INSTALL_FEE: u128 = 10u128;
pub const ZERO_WALLET_FEE: u128 = 0u128;
pub const ZERO_REGISTRY_FEE: u128 = 0u128;
pub const ZERO_INSTALL_FEE: u128 = 0u128;
pub const MINTER_CAP: u128 = 10000;

// Indexes
pub const IDEPLOYER: usize = 0;
pub const ICOMMITTEE: usize = 1;
pub const IRELAYER: usize = 2;
