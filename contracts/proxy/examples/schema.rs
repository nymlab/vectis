use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::{CosmosMsg, Empty};

use cw1::CanExecuteResponse;
use vectis_proxy::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use vectis_wallet::{Nonce, RelayTransaction, WalletInfo};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Nonce), &out_dir);
    export_schema(&schema_for!(RelayTransaction), &out_dir);
    export_schema_with_title(&schema_for!(ExecuteMsg<Empty>), &out_dir, "ExecuteMsg");
    export_schema_with_title(&schema_for!(WalletInfo), &out_dir, "InfoResponse");
    export_schema(&schema_for!(Empty), &out_dir);
    export_schema(&schema_for!(CosmosMsg<Empty>), &out_dir);
    // export_schema_with_title(
    //     &schema_for!(CosmosMsg<Empty>),
    //     &out_dir,
    //     "CosmosMsg_for_empty",
    // );

    export_schema_with_title(
        &schema_for!(CanExecuteResponse),
        &out_dir,
        "CanExecuteRelayResponse",
    );
}
