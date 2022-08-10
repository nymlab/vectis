/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.10.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

export interface CanExecuteRelayResponse {
    can_execute: boolean;
    [k: string]: unknown;
}
export type CosmosMsgForEmpty =
    | {
          bank: BankMsg;
      }
    | {
          custom: Empty;
      }
    | {
          staking: StakingMsg;
      }
    | {
          distribution: DistributionMsg;
      }
    | {
          wasm: WasmMsg;
      };
export type BankMsg =
    | {
          send: {
              amount: Coin[];
              to_address: string;
              [k: string]: unknown;
          };
      }
    | {
          burn: {
              amount: Coin[];
              [k: string]: unknown;
          };
      };
export type Uint128 = string;
export type StakingMsg =
    | {
          delegate: {
              amount: Coin;
              validator: string;
              [k: string]: unknown;
          };
      }
    | {
          undelegate: {
              amount: Coin;
              validator: string;
              [k: string]: unknown;
          };
      }
    | {
          redelegate: {
              amount: Coin;
              dst_validator: string;
              src_validator: string;
              [k: string]: unknown;
          };
      };
export type DistributionMsg =
    | {
          set_withdraw_address: {
              address: string;
              [k: string]: unknown;
          };
      }
    | {
          withdraw_delegator_reward: {
              validator: string;
              [k: string]: unknown;
          };
      };
export type WasmMsg =
    | {
          execute: {
              contract_addr: string;
              funds: Coin[];
              msg: Binary;
              [k: string]: unknown;
          };
      }
    | {
          instantiate: {
              admin?: string | null;
              code_id: number;
              funds: Coin[];
              label: string;
              msg: Binary;
              [k: string]: unknown;
          };
      }
    | {
          migrate: {
              contract_addr: string;
              msg: Binary;
              new_code_id: number;
              [k: string]: unknown;
          };
      }
    | {
          update_admin: {
              admin: string;
              contract_addr: string;
              [k: string]: unknown;
          };
      }
    | {
          clear_admin: {
              contract_addr: string;
              [k: string]: unknown;
          };
      };
export type Binary = string;
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export interface Empty {
    [k: string]: unknown;
}
export type ExecuteMsgForEmpty =
    | {
          execute: {
              msgs: CosmosMsgForEmpty[];
              [k: string]: unknown;
          };
      }
    | {
          revert_freeze_status: {
              [k: string]: unknown;
          };
      }
    | {
          relay: {
              transaction: RelayTransaction;
              [k: string]: unknown;
          };
      }
    | {
          rotate_user_key: {
              new_user_address: string;
              [k: string]: unknown;
          };
      }
    | {
          add_relayer: {
              new_relayer_address: Addr;
              [k: string]: unknown;
          };
      }
    | {
          remove_relayer: {
              relayer_address: Addr;
              [k: string]: unknown;
          };
      }
    | {
          update_guardians: {
              guardians: Guardians;
              new_multisig_code_id?: number | null;
              [k: string]: unknown;
          };
      }
    | {
          update_label: {
              new_label: string;
              [k: string]: unknown;
          };
      };
export type Addr = string;
export interface RelayTransaction {
    message: Binary;
    nonce: number;
    signature: Binary;
    user_pubkey: Binary;
    [k: string]: unknown;
}
export interface Guardians {
    addresses: string[];
    guardians_multisig?: MultiSig | null;
    [k: string]: unknown;
}
export interface MultiSig {
    multisig_initial_funds: Coin[];
    threshold_absolute_count: number;
    [k: string]: unknown;
}
export interface InfoResponse {
    code_id: number;
    guardians: Addr[];
    is_frozen: boolean;
    label: string;
    multisig_address?: Addr | null;
    multisig_code_id: number;
    nonce: number;
    relayers: Addr[];
    user_addr: Addr;
    version: ContractVersion;
    [k: string]: unknown;
}
export interface ContractVersion {
    contract: string;
    version: string;
    [k: string]: unknown;
}
export interface InstantiateMsg {
    addr_prefix: string;
    code_id: number;
    create_wallet_msg: CreateWalletMsg;
    multisig_code_id: number;
    [k: string]: unknown;
}
export interface CreateWalletMsg {
    guardians: Guardians;
    label: string;
    proxy_initial_funds: Coin[];
    relayers: string[];
    user_addr: string;
    [k: string]: unknown;
}
export type QueryMsg =
    | {
          info: {
              [k: string]: unknown;
          };
      }
    | {
          can_execute_relay: {
              sender: string;
              [k: string]: unknown;
          };
      };
export type Uint64 = number;
