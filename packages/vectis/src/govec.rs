use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct StakingOptions {
    pub duration: Option<Duration>,
    pub code_id: u64,
}
