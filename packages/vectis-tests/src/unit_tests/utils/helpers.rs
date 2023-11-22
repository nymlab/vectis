use crate::unit_tests::utils::*;

pub fn contract_flex_multisig() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        flex_multisig_execute,
        flex_multisig_instantiate,
        flex_multisig_query,
    );
    Box::new(contract)
}

pub fn contract_cw4() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(cw4_execute, cw4_instantiate, cw4_query);
    Box::new(contract)
}

pub fn add_item_msg(key: VectisActors, value: Addr) -> cw3flexExecMsg {
    cw3flexExecMsg::UpdateItem {
        key: format!("{key}"),
        value: value.to_string(),
    }
}
