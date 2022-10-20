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
    pub fn verify_guardians(&self, user: &Addr) -> StdResult<()> {
        for g in &self.addresses {
            if g == user.as_str() {
                return Err(StdError::generic_err("user cannot be a guardian"));
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
/// This contract can be an instance of 3 code ids.
/// - 1: exisiting stored `MULTISIG_CODE_ID` if `new_multisig_code_id == None`
/// - 2: the `new_multisig_code_id` if given
/// - 3: if 1 nor 2 are available, the supported multisig from the FACTORY will be used.
#[cw_serde]
pub struct GuardiansUpdateMsg {
    pub guardians: Guardians,
    pub new_multisig_code_id: Option<u64>,
}
