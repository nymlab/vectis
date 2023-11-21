pub use cosmwasm_std::{
    coin, instantiate2_address, to_binary, Addr, Api, BlockInfo, CanonicalAddr, Coin,
    RecoverPubkeyError, StdError, StdResult, Storage, Timestamp, Uint128, VerificationError,
};
pub use cw_multi_test::{error::AnyResult, AddressGenerator, AppBuilder, BankKeeper, WasmKeeper};
pub use sylvia::multitest::App;

pub use vectis_wallet::types::{
    authenticator::{Authenticator, AuthenticatorProvider, AuthenticatorType, EmptyInstantiateMsg},
    entity::Entity,
    factory::{
        AuthenticatorInstInfo, ChainConnection, CodeIdType, FeeType, FeesResponse,
        WalletFactoryInstantiateMsg,
    },
    plugin_registry::{SubscriptionTier, TierDetails},
};

pub use vectis_factory::management::contract::sv::test_utils::*;
pub use vectis_plugin_registry::management::contract::sv::test_utils::RegistryManagementTrait;

pub use vectis_factory::contract::sv::multitest_utils::CodeId as FactoryCodeId;
pub use vectis_plugin_registry::contract::sv::multitest_utils::CodeId as RegistryCodeId;
pub use vectis_proxy::contract::sv::multitest_utils::CodeId as ProxyCodeId;
pub use vectis_webauthn_authenticator::contract::sv::multitest_utils::CodeId as AuthCodeId;

pub use bech32::{decode, encode, FromBase32, ToBase32, Variant};
pub use sha2::{digest::Update, Digest, Sha256};

pub use crate::constants::*;

pub fn default_entity() -> Entity {
    Entity {
        auth: Authenticator {
            ty: AuthenticatorType::Webauthn,
            provider: AuthenticatorProvider::Vectis,
        },
        data: to_binary(&"data").unwrap(),
        nonce: 0,
    }
}
