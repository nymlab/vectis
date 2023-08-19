use crate::types::error::PluginRegError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{CanonicalAddr, Deps};
use std::collections::BTreeMap;

#[cw_serde]
pub struct PluginInfo {
    pub src: PluginSource,
    pub version: String,
}

/// Permission of the plugin on the proxy
#[cw_serde]
pub enum PluginPermissions {
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
    VectisRegistry(u64),
    /// This is the code_id and the version of the unregistered plugin
    CodeId(u64, String),
}

impl std::fmt::Display for PluginSource {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cw_serde]
pub struct PluginParams {
    pub permission: PluginPermissions,
}

#[cw_serde]
pub struct VersionDetails {
    /// Uploaded Plugin code id
    pub code_id: u64,
    /// Should be checked upon instantiation
    pub checksum: String,
    /// Useful for storing display data
    pub ipfs_hash: String,
}

#[cw_serde]
pub struct Plugin {
    /// Identifier of the plugin, does not change over time
    pub id: u64,
    /// Name of the Plugin, does not change over time
    pub name: String,
    /// Reference Addr onchain to the creator
    pub creator: CanonicalAddr,
    /// Latest version of the plugin
    pub latest_version: String,
    /// Mapping of all versions to the details
    pub versions: BTreeMap<String, VersionDetails>,
}

impl Plugin {
    pub fn get_latest_version_details(&self) -> Result<VersionDetails, PluginRegError> {
        self.versions
            .get(&self.latest_version)
            .map(|l| l.to_owned())
            .ok_or_else(|| PluginRegError::PluginVersionNotFound(self.latest_version.clone()))
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
        name: String,
        creator: String,
        ipfs_hash: String,
        version: String,
        code_id: u64,
        checksum: String,
    ) -> Result<Plugin, PluginRegError> {
        let mut record = BTreeMap::new();
        record.insert(
            version.clone(),
            VersionDetails {
                code_id,
                checksum,
                ipfs_hash,
            },
        );

        Ok(Plugin {
            id,
            name,
            creator: deps.api.addr_canonicalize(&creator)?,
            latest_version: version,
            versions: record,
        })
    }
}
