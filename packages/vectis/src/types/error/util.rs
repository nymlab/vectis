use super::*;

/// Relay transaction related errors
#[derive(Error, Debug, PartialEq)]
pub enum DeployerItemsQueryError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Item not set for {0}")]
    ItemNotSet(String),
    #[error("Deployer Addr Not Found")]
    DeployerAddrNotFound,
}

#[derive(Error, Debug, PartialEq)]
pub enum Inst2CalcError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Instantiate2(#[from] Instantiate2AddressError),
}
