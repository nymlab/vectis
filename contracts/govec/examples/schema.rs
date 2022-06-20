use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::{Addr, Binary};
use cw20::{AllAccountsResponse, BalanceResponse, TokenInfoResponse, DownloadLogoResponse};
use vectis_govec::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::MinterData,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(BalanceResponse), &out_dir);
    export_schema(&schema_for!(TokenInfoResponse), &out_dir);
    export_schema(&schema_for!(DownloadLogoResponse), &out_dir);
    export_schema(&schema_for!(AllAccountsResponse), &out_dir);
    export_schema(&schema_for!(MinterData), &out_dir);
    export_schema(&schema_for!(Binary), &out_dir);
    export_schema_with_title(&schema_for!(MinterData), &out_dir, "MinterResponse");
    export_schema_with_title(&schema_for!(Addr), &out_dir, "StakingResponse");
    export_schema_with_title(&schema_for!(Addr), &out_dir, "DaoResponse");
}
