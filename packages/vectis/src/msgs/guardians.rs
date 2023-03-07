use std::ops::Add;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, StdError, StdResult};
use cw_utils::{Duration, Expiration, DAY};

use crate::MultiSig;

/// Min delay time is set to 1 day
pub const GUARDIAN_REQUEST_ACTIVATION_TIME: Duration = DAY;

#[cw_serde]
pub struct Guardians {
    /// A List of keys can act as guardian for
    pub addresses: Vec<String>,
    /// Whether multisig option for guardians is enabled
    pub guardians_multisig: Option<MultiSig>,
}

impl Guardians {
    pub fn verify_guardians(&self, controller: &Addr) -> StdResult<()> {
        for g in &self.addresses {
            if g == controller.as_str() {
                return Err(StdError::generic_err("controller cannot be a guardian"));
            }
        }
        Ok(())
    }
}

#[cw_serde]
pub struct GuardiansUpdateRequest {
    pub guardians: Guardians,
    pub new_multisig_code_id: Option<u64>,
    pub activate_at: Expiration,
}

impl GuardiansUpdateRequest {
    pub fn new(
        guardians: Guardians,
        new_multisig_code_id: Option<u64>,
        block: &BlockInfo,
    ) -> GuardiansUpdateRequest {
        let activate_at = Expiration::AtTime(block.time)
            .add(GUARDIAN_REQUEST_ACTIVATION_TIME)
            .expect("error defining activate_at");

        GuardiansUpdateRequest {
            guardians,
            new_multisig_code_id,
            activate_at,
        }
    }
}

/// If the `Guardians.guardian_multisig` is given,
/// we will instantiate a new multisig contract.
/// This contract can be an instance of 2 code ids.
/// - 1: the `new_multisig_code_id` if given, OR
/// - 2: the supported multisig from the facotry will be used.
#[cw_serde]
pub struct GuardiansUpdateMsg {
    pub guardians: Guardians,
    pub new_multisig_code_id: Option<u64>,
}
