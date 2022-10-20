/**
 * This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run the @cosmwasm/ts-codegen generate command to regenerate this file.
 */

export type Uint128 = string;
export interface InstantiateMsg {
    addr_prefix: string;
    govec_minter?: string | null;
    proxy_code_id: number;
    proxy_multisig_code_id: number;
    wallet_fee: Coin;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export type ExecuteMsg =
    | {
          create_wallet: {
              create_wallet_msg: CreateWalletMsg;
          };
      }
    | {
          migrate_wallet: {
              migration_msg: ProxyMigrationTxMsg;
              wallet_address: WalletAddr;
          };
      }
    | {
          update_code_id: {
              new_code_id: number;
              ty: CodeIdType;
          };
      }
    | {
          update_wallet_fee: {
              new_fee: Coin;
          };
      }
    | {
          update_govec_addr: {
              addr: string;
          };
      }
    | {
          update_dao: {
              addr: string;
          };
      }
    | {
          claim_govec: {};
      }
    | {
          govec_minted: {
              wallet: string;
          };
      }
    | {
          purge_expired_claims: {
              limit?: number | null;
              start_after?: string | null;
          };
      };
export type ProxyMigrationTxMsg =
    | {
          relay_tx: RelayTransaction;
      }
    | {
          direct_migration_msg: Binary;
      };
export type Binary = string;
export type WalletAddr =
    | {
          canonical: CanonicalAddr;
      }
    | {
          addr: Addr;
      };
export type CanonicalAddr = string;
export type Addr = string;
export type CodeIdType = "proxy" | "multisig";
export interface CreateWalletMsg {
    guardians: Guardians;
    label: string;
    proxy_initial_funds: Coin[];
    relayers: string[];
    user_addr: string;
}
export interface Guardians {
    addresses: string[];
    guardians_multisig?: MultiSig | null;
}
export interface MultiSig {
    multisig_initial_funds: Coin[];
    threshold_absolute_count: number;
}
export interface RelayTransaction {
    message: Binary;
    nonce: number;
    signature: Binary;
    user_pubkey: Binary;
}
export type QueryMsg =
    | {
          unclaimed_govec_wallets: {
              limit?: number | null;
              start_after?: string | null;
          };
      }
    | {
          claim_expiration: {
              wallet: string;
          };
      }
    | {
          total_created: {};
      }
    | {
          code_id: {
              ty: CodeIdType;
          };
      }
    | {
          fee: {};
      }
    | {
          govec_addr: {};
      }
    | {
          dao_addr: {};
      };
export type Expiration =
    | {
          at_height: number;
      }
    | {
          at_time: Timestamp;
      }
    | {
          never: {
              [k: string]: unknown;
          };
      };
export type Timestamp = Uint64;
export type Uint64 = string;
export interface UnclaimedWalletList {
    wallets: [Addr, Expiration][];
}
