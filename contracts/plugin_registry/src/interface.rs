use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdError};
use sylvia::{interface, schemars};

#[interface(module=installable)]
pub trait Installable {
    type Error: From<StdError>;

    #[msg(exec)]
    fn proxy_install_plugin(
        &self,
        ctx: (DepsMut, Env, MessageInfo),
        id: u64,
        instantiate_msg: Binary,
    ) -> Result<Response, Self::Error>;
}
