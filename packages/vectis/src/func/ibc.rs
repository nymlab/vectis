use cosmwasm_std::IbcOrder;

use crate::{IbcError, IBC_APP_ORDER, IBC_APP_VERSION};

pub fn check_ibc_order(order: &IbcOrder) -> Result<(), IbcError> {
    if order != &IBC_APP_ORDER {
        Err(IbcError::InvalidChannelOrder)
    } else {
        Ok(())
    }
}

pub fn check_ibc_version(version: &str) -> Result<(), IbcError> {
    if version != IBC_APP_VERSION {
        Err(IbcError::InvalidChannelVersion(IBC_APP_VERSION))
    } else {
        Ok(())
    }
}
