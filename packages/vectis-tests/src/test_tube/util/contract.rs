use cosmwasm_std::{to_binary, Coin, CosmosMsg, WasmMsg};
use osmosis_std::types::cosmwasm::wasm::v1::MsgExecuteContractResponse;
use osmosis_test_tube::{
    Account, Module, OsmosisTestApp, Runner, RunnerError, RunnerExecuteResult, RunnerResult,
    SigningAccount, Wasm,
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug)]
pub struct Contract<'a> {
    pub app: &'a OsmosisTestApp,
    pub contract_addr: String,
    pub code_id: Option<u64>,
}

impl<'a> Contract<'a> {
    pub fn from_addr(app: &'a OsmosisTestApp, contract_addr: String) -> Self {
        Self {
            app,
            contract_addr,
            code_id: None,
        }
    }

    pub fn store_code(app: &'a OsmosisTestApp, code_path: &str, signer: &SigningAccount) -> u64 {
        let wasm = Wasm::new(app);
        let wasm_byte_code = std::fs::read(code_path).unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, signer)
            .unwrap()
            .data
            .code_id;
        code_id
    }

    pub fn deploy<M>(
        app: &'a OsmosisTestApp,
        code_path: &str,
        instantiate_msg: &M,
        signer: &SigningAccount,
    ) -> Result<Self, RunnerError>
    where
        M: ?Sized + Serialize,
    {
        let wasm = Wasm::new(app);
        let wasm_byte_code = std::fs::read(code_path).unwrap();
        let code_id = wasm.store_code(&wasm_byte_code, None, signer)?.data.code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &instantiate_msg,
                Some(&signer.address()),
                None,
                &[],
                signer,
            )?
            .data
            .address;

        Ok(Self {
            app,
            code_id: Some(code_id),
            contract_addr,
        })
    }

    pub fn execute<M>(
        &self,
        msg: &M,
        funds: &[Coin],
        signer: &SigningAccount,
    ) -> RunnerExecuteResult<MsgExecuteContractResponse>
    where
        M: ?Sized + Serialize,
    {
        let execute_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: self.contract_addr.clone(),
            msg: to_binary(&msg).unwrap(),
            funds: funds.to_vec(),
        });
        self.app
            .execute_cosmos_msgs::<MsgExecuteContractResponse>(&[execute_msg], &signer)
    }

    pub fn query<T, M>(&self, msg: &M) -> RunnerResult<T>
    where
        M: ?Sized + Serialize,
        T: ?Sized + DeserializeOwned,
    {
        let wasm = Wasm::new(self.app);
        wasm.query(&self.contract_addr, msg)
    }
}
