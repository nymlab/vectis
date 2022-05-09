use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::{Addr, Coin};
use cw20::Cw20Coin;
use vectis_factory::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, WalletListResponse};
use vectis_factory::state::WalletInfo;
use vectis_wallet::{
    CodeIdType, CreateWalletMsg, ProxyMigrationTxMsg, StakingOptions, WalletAddr, WalletQueryPrefix,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(WalletInfo), &out_dir);
    export_schema(&schema_for!(CreateWalletMsg), &out_dir);
    export_schema(&schema_for!(CodeIdType), &out_dir);
    export_schema(&schema_for!(WalletQueryPrefix), &out_dir);
    export_schema(&schema_for!(ProxyMigrationTxMsg), &out_dir);
    export_schema(&schema_for!(WalletAddr), &out_dir);
    export_schema(&schema_for!(Cw20Coin), &out_dir);
    export_schema(&schema_for!(StakingOptions), &out_dir);
    export_schema_with_title(&schema_for!(QueryMsg), &out_dir, "QueryMsg");
    export_schema_with_title(&schema_for!(u64), &out_dir, "CodeIdResponse");
    export_schema_with_title(&schema_for!(Coin), &out_dir, "FeeResponse");
    export_schema_with_title(&schema_for!(Addr), &out_dir, "GovecAddrResponse");
    export_schema_with_title(&schema_for!(Addr), &out_dir, "AdminAddrResponse");
    export_schema_with_title(
        &schema_for!(WalletListResponse),
        &out_dir,
        "WalletsOfResponse",
    );
    export_schema_with_title(
        &schema_for!(WalletListResponse),
        &out_dir,
        "WalletsResponse",
    );
}
