mod authenticator;
mod factory;
mod plugin_registry;
mod wallet;

pub use {
    authenticator::{authenticator_trait, authenticator_trait::AuthenticatorTrait},
    factory::{
        factory_management_trait, factory_management_trait::FactoryManagementTrait,
        factory_service_trait, factory_service_trait::FactoryServiceTrait,
    },
    plugin_registry::{
        registry_management_trait, registry_management_trait::RegistryManagementTrait,
        registry_service_trait, registry_service_trait::RegistryServiceTrait,
    },
    wallet::{
        wallet_plugin_trait, wallet_plugin_trait::WalletPluginTrait, wallet_trait,
        wallet_trait::WalletTrait,
    },
};