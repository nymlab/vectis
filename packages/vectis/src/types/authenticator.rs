use crate::Nonce;
use sylvia::cw_std::StdError;
use sylvia::interface;
use sylvia::types::ExecCtx;

#[interface]
pub trait Authenicator {
    type Error: From<StdError>;

    #[msg(query)]
    fn authenticate(
        &self,
        ctx: QueryCtx,
        auth_data: &Binary,
        nonce: Nonce,
    ) -> Result<bool, Self::Error>;
}
