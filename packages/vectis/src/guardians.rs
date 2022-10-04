use std::ops::Add;

use schemars::JsonSchema;
use cosmwasm_std::{Addr, StdError, StdResult, BlockInfo};
use serde::{Deserialize, Serialize};

use crate::MultiSig;

/// Min delay time is set to >= 24 hours
const MIN_DELAY_TIME: u64 = 60 * 60 * 24;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Guardians {
    /// A List of keys can act as guardian for
    pub addresses: Vec<String>,
    /// Whether multisig option for guardians is enabled
    pub guardians_multisig: Option<MultiSig>,
}

impl Guardians {
    pub fn verify_guardians(&self, user: &Addr) -> StdResult<()> {
        for g in &self.addresses {
            if g == user.as_str() {
                return Err(StdError::generic_err("user cannot be a guardian"));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct GuardiansUpdateRequest {
    pub guardians: Guardians,
    pub new_multisig_code_id: Option<u64>,
    /// Creation time in seconds
    pub created_at: u64,
}

impl GuardiansUpdateRequest {
    pub fn ensure_delay_passed(&self, block: &BlockInfo) -> StdResult<()> {
        let delay = self.created_at.add(MIN_DELAY_TIME);
        if delay <= block.time.seconds() {
            return Err(StdError::generic_err("it is necessary to wait for this execution"))
        }
        Ok(())
    }
}

/// If the `Guardians.guardian_multisig` is given,
/// we will instantiate a new multisig contract.
/// This contract can be an instance of 3 code ids.
/// - 1: exisiting stored `MULTISIG_CODE_ID` if `new_multisig_code_id == None`
/// - 2: the `new_multisig_code_id` if given
/// - 3: if 1 nor 2 are available, the supported multisig from the FACTORY will be used.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct GuardiansUpdateMsg {
    pub guardians: Guardians,
    pub new_multisig_code_id: Option<u64>,
}