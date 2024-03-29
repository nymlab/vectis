/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.35.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export type Binary = string;
export type AuthenticatorType = "webauthn" | {
  other: string;
};
export type ChainConnection = {
  i_b_c: string;
} | {
  other: string;
};
export type Uint128 = string;
export interface InstantiateMsg {
  msg: WalletFactoryInstantiateMsg;
  [k: string]: unknown;
}
export interface WalletFactoryInstantiateMsg {
  authenticators?: AuthenticatorInstInfo[] | null;
  default_proxy_code_id: number;
  supported_chains?: [string, ChainConnection][] | null;
  supported_proxies: [number, string][];
  wallet_creator: string;
  wallet_fee: Coin;
}
export interface AuthenticatorInstInfo {
  code_id: number;
  inst_msg: Binary;
  ty: AuthenticatorType;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export type ExecuteMsg = FactoryManagementTraitExecMsg | FactoryServiceTraitExecMsg | ExecMsg;
export type FactoryManagementTraitExecMsg = {
  update_code_id: {
    code_id: number;
    set_as_default: boolean;
    ty: CodeIdType;
    version?: string | null;
    [k: string]: unknown;
  };
} | {
  update_config_fee: {
    new_fee: Coin;
    ty: FeeType;
    [k: string]: unknown;
  };
} | {
  update_supported_interchain: {
    chain_connection?: ChainConnection | null;
    chain_id: string;
    [k: string]: unknown;
  };
} | {
  update_deployer: {
    addr: string;
    [k: string]: unknown;
  };
} | {
  update_wallet_creator: {
    addr: string;
    [k: string]: unknown;
  };
} | {
  update_auth_provider: {
    new_code_id?: number | null;
    new_inst_msg?: Binary | null;
    ty: AuthenticatorType;
    [k: string]: unknown;
  };
};
export type CodeIdType = "proxy";
export type FeeType = "wallet";
export type FactoryServiceTraitExecMsg = {
  create_wallet: {
    create_wallet_msg: CreateWalletMsg;
    [k: string]: unknown;
  };
} | {
  migrate_wallet: {
    migrations_msg: MigrateWalletMsg;
    [k: string]: unknown;
  };
};
export type AuthenticatorProvider = "vectis" | {
  custom: string;
};
export type PluginPermission = "exec" | "pre_tx_check" | "post_tx_hook";
export type PluginSource = {
  vectis_registry: [number, string | null];
};
export type ExecMsg = string;
export interface CreateWalletMsg {
  chains?: [string, string][] | null;
  code_id?: number | null;
  controller: Entity;
  initial_data: [Binary, Binary][];
  plugins: PluginInstallParams[];
  proxy_initial_funds: Coin[];
  relayers: string[];
  vid: string;
}
export interface Entity {
  auth: Authenticator;
  data: Binary;
  nonce: number;
}
export interface Authenticator {
  provider: AuthenticatorProvider;
  ty: AuthenticatorType;
}
export interface PluginInstallParams {
  funds: Coin[];
  instantiate_msg: Binary;
  label: string;
  permission: PluginPermission;
  src: PluginSource;
}
export interface MigrateWalletMsg {
  addr_to_migrate: string;
  tx: RelayTransaction;
}
export interface RelayTransaction {
  message: Binary;
  signature: Binary;
}
export type QueryMsg = FactoryManagementTraitQueryMsg | FactoryServiceTraitQueryMsg | QueryMsg1;
export type FactoryManagementTraitQueryMsg = {
  total_created: {
    [k: string]: unknown;
  };
} | {
  default_proxy_code_id: {
    [k: string]: unknown;
  };
} | {
  deployer: {
    [k: string]: unknown;
  };
} | {
  wallet_creator: {
    [k: string]: unknown;
  };
} | {
  supported_chains: {
    limit?: number | null;
    start_after?: string | null;
    [k: string]: unknown;
  };
} | {
  supported_proxies: {
    limit?: number | null;
    start_after?: number | null;
    [k: string]: unknown;
  };
} | {
  fees: {
    [k: string]: unknown;
  };
} | {
  auth_provider_addr: {
    ty: AuthenticatorType;
    [k: string]: unknown;
  };
} | {
  contract_version: {
    [k: string]: unknown;
  };
};
export type FactoryServiceTraitQueryMsg = {
  wallet_by_vid: {
    vid: string;
    [k: string]: unknown;
  };
} | {
  wallet_by_vid_chain: {
    chain_id: string;
    vid: string;
    [k: string]: unknown;
  };
};
export type QueryMsg1 = string;
export type NullableAddr = Addr | null;
export type Addr = string;
export interface ContractVersion {
  contract: string;
  version: string;
}
export type Uint64 = number;
export interface FeesResponse {
  wallet_fee: Coin;
}
export type ArrayOfTupleOfStringAndChainConnection = [string, ChainConnection][];
export type ArrayOfTupleOfUint64AndString = [number, string][];
export type NullableString = string | null;