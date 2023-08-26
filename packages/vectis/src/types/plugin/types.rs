use crate::types::error::PluginRegError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, CanonicalAddr, Coin, Deps};
use std::collections::BTreeMap;

/// Input for migrating a plugin
#[cw_serde]
pub struct PluginMigrateParams {
    /// The existing plugin addr to migrate
    pub plugin_addr: String,
    /// The existing plugin permission; this will not change in the migration
    /// It is included here to help the contract find the plugin
    pub plugin_permission: PluginPermission,
    /// The new src; this should only be different to existing if it is not vectis registry
    pub target_src: PluginSource,
    pub migration_msg: Binary,
    pub funds: Vec<Coin>,
}

/// Input for installing a plugin
#[cw_serde]
pub struct PluginInstallParams {
    pub src: PluginSource,
    pub instantiate_msg: Binary,
    pub permission: PluginPermission,
    pub label: String,
    pub funds: Vec<Coin>,
}

#[cw_serde]
pub struct PluginCodeData {
    pub latest_contract_version: String, // Must update version to the cw2 contract version
    pub new_code_id: u64,                // Version must match new code
    pub new_code_hash: String,           // Code_id must point to this code_hash
}

#[cw_serde]
pub struct PluginMetadataData {
    pub creator: String,
    pub display_name: String,
    pub ipfs_hash: String,
}

#[cw_serde]
pub struct PluginInfo {
    pub src: PluginSource,
    pub version: String,
    pub permission: PluginPermission,
}

/// Permission of the plugin on the proxy
#[cw_serde]
pub enum PluginPermission {
    /// Can Exec through Proxy
    Exec,
    /// Is used to check tx before execution
    // TODO: Impl
    PreTxCheck,
    /// Hooks post tx
    // TODO: Impl
    PostTxHook,
}

/// The source of the plugin code.
#[cw_serde]
pub enum PluginSource {
    /// Plugin registry id and version (latest if not provided)
    VectisRegistry(u64, Option<String>),
    /// This is the code_id and the version of the unregistered plugin
    CodeId(u64, String),
}

impl std::fmt::Display for PluginSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cw_serde]
pub struct VersionDetails {
    /// Uploaded Plugin code id
    pub code_id: u64,
    /// code_hash of the contract
    pub code_hash: String,
    /// Useful for storing display data
    pub ipfs_hash: String,
}

#[cw_serde]
pub struct Plugin {
    /// Identifier of the plugin, does not change over time
    pub id: u64,
    /// Reference Addr onchain to the creator
    pub creator: CanonicalAddr,
    /// Display name, creator can define this
    pub display_name: String,
    /// Latest cw2 contract version
    pub latest_contract_version: String,
    /// Mapping of all versions to the details
    pub versions: BTreeMap<String, VersionDetails>,
}

impl Plugin {
    pub fn get_latest_version_details(&self) -> Result<VersionDetails, PluginRegError> {
        self.versions
            .get(&self.latest_contract_version)
            .map(|l| l.to_owned())
            .ok_or_else(|| {
                PluginRegError::PluginVersionNotFound(self.latest_contract_version.clone())
            })
    }

    pub fn get_version_details(&self, version: &str) -> Result<VersionDetails, PluginRegError> {
        self.versions
            .get(version)
            .map(|l| l.to_owned())
            .ok_or_else(|| PluginRegError::PluginVersionNotFound(version.to_string()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        deps: Deps,
        id: u64,
        creator: String,
        display_name: String,
        ipfs_hash: String,
        latest_contract_version: String,
        code_id: u64,
        code_hash: String,
    ) -> Result<Plugin, PluginRegError> {
        let mut record = BTreeMap::new();
        record.insert(
            latest_contract_version.clone(),
            VersionDetails {
                code_id,
                code_hash,
                ipfs_hash,
            },
        );

        Ok(Plugin {
            id,
            creator: deps.api.addr_canonicalize(&creator)?,
            display_name,
            latest_contract_version,
            versions: record,
        })
    }
}
