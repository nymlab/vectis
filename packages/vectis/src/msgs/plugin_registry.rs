use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct RegisterPlugin {
    pub name: String,
    pub creator: String,
    pub ipfs_hash: String,
    pub version: String,
    pub code_id: u64,
    pub checksum: String,
}
