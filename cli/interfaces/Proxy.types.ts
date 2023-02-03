/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.24.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

export type Uint128 = string;
export interface InstantiateMsg {
    code_id: number;
    create_wallet_msg: CreateWalletMsg;
    multisig_code_id: number;
}
export interface CreateWalletMsg {
    controller_addr: string;
    guardians: Guardians;
    label: string;
    proxy_initial_funds: Coin[];
    relayers: string[];
}
export interface Guardians {
    addresses: string[];
    guardians_multisig?: MultiSig | null;
}
export interface MultiSig {
    multisig_initial_funds: Coin[];
    threshold_absolute_count: number;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export type ExecuteMsg =
    | {
          execute: {
              msgs: CosmosMsgForEmpty[];
          };
      }
    | {
          revert_freeze_status: {};
      }
    | {
          relay: {
              transaction: RelayTransaction;
          };
      }
    | {
          rotate_controller_key: {
              new_controller_address: string;
          };
      }
    | {
          add_relayer: {
              new_relayer_address: Addr;
          };
      }
    | {
          remove_relayer: {
              relayer_address: Addr;
          };
      }
    | {
          request_update_guardians: {
              request?: GuardiansUpdateMsg | null;
          };
      }
    | {
          update_guardians: {};
      }
    | {
          update_label: {
              new_label: string;
          };
      }
    | {
          instantiate_plugin: {
              code_id: number;
              instantiate_msg: Binary;
              label: string;
              plugin_params: PluginParams;
          };
      }
    | {
          update_plugins: {
              migrate_msg?: [number, Binary] | null;
              plugin_addr: string;
          };
      }
    | {
          plugin_execute: {
              msgs: CosmosMsgForEmpty[];
          };
      };
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
          stargate: {
              type_url: string;
              value: Binary;
              [k: string]: unknown;
          };
      }
    | {
          ibc: IbcMsg;
      }
    | {
          wasm: WasmMsg;
      }
    | {
          gov: GovMsg;
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
export type Binary = string;
export type IbcMsg =
    | {
          transfer: {
              amount: Coin;
              channel_id: string;
              timeout: IbcTimeout;
              to_address: string;
              [k: string]: unknown;
          };
      }
    | {
          send_packet: {
              channel_id: string;
              data: Binary;
              timeout: IbcTimeout;
              [k: string]: unknown;
          };
      }
    | {
          close_channel: {
              channel_id: string;
              [k: string]: unknown;
          };
      };
export type Timestamp = Uint64;
export type Uint64 = string;
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
export type GovMsg = {
    vote: {
        proposal_id: number;
        vote: VoteOption;
        [k: string]: unknown;
    };
};
export type VoteOption = "yes" | "no" | "abstain" | "no_with_veto";
export type Addr = string;
export interface Empty {
    [k: string]: unknown;
}
export interface IbcTimeout {
    block?: IbcTimeoutBlock | null;
    timestamp?: Timestamp | null;
    [k: string]: unknown;
}
export interface IbcTimeoutBlock {
    height: number;
    revision: number;
    [k: string]: unknown;
}
export interface RelayTransaction {
    controller_pubkey: Binary;
    message: Binary;
    nonce: number;
    signature: Binary;
}
export interface GuardiansUpdateMsg {
    guardians: Guardians;
    new_multisig_code_id?: number | null;
}
export interface PluginParams {
    grantor: boolean;
}
export type QueryMsg =
    | {
          info: {};
      }
    | {
          can_execute_relay: {
              sender: string;
          };
      }
    | {
          guardians_update_request: {};
      }
    | {
          plugins: {
              limit?: number | null;
              start_after?: string | null;
          };
      };
export interface CanExecuteResponse {
    can_execute: boolean;
}
export type NullableGuardiansUpdateRequest = GuardiansUpdateRequest | null;
export type Expiration =
    | {
          at_height: number;
      }
    | {
          at_time: Timestamp;
      }
    | {
          never: {};
      };
export interface GuardiansUpdateRequest {
    activate_at: Expiration;
    guardians: Guardians;
    new_multisig_code_id?: number | null;
}
export interface WalletInfo {
    code_id: number;
    controller_addr: Addr;
    factory: Addr;
    guardians: Addr[];
    is_frozen: boolean;
    label: string;
    multisig_address?: Addr | null;
    multisig_code_id: number;
    nonce: number;
    relayers: Addr[];
    version: ContractVersion;
}
export interface ContractVersion {
    contract: string;
    version: string;
}
export interface PluginListResponse {
    plugins: Addr[];
}
