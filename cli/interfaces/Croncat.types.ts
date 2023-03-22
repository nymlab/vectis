export type Addr = string;
export interface InstantiateMsg {
    chain_name: string;
    croncat_agents_key: [string, [number, number]];
    croncat_manager_key: [string, [number, number]];
    gas_action_fee?: number | null;
    gas_base_fee?: number | null;
    gas_limit?: number | null;
    gas_query_fee?: number | null;
    pause_admin: Addr;
    slot_granularity_time?: number | null;
    version?: string | null;
}
export type ExecuteMsg =
    | {
          update_config: UpdateConfigMsg;
      }
    | {
          create_task: {
              task: TaskRequest;
          };
      }
    | {
          remove_task: {
              task_hash: string;
          };
      }
    | {
          remove_task_by_manager: TasksRemoveTaskByManager;
      }
    | {
          reschedule_task: TasksRescheduleTask;
      }
    | {
          pause_contract: {};
      }
    | {
          unpause_contract: {};
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
export type Boundary =
    | {
          height: BoundaryHeight;
      }
    | {
          time: BoundaryTime;
      };
export type Interval =
    | "once"
    | "immediate"
    | {
          block: number;
      }
    | {
          cron: string;
      };
export type ValueIndex =
    | {
          key: string;
      }
    | {
          index: number;
      };
export type PathToValue = ValueIndex[];
export interface UpdateConfigMsg {
    croncat_agents_key?: [string, [number, number]] | null;
    croncat_manager_key?: [string, [number, number]] | null;
    gas_action_fee?: number | null;
    gas_base_fee?: number | null;
    gas_limit?: number | null;
    gas_query_fee?: number | null;
    slot_granularity_time?: number | null;
}
export interface TaskRequest {
    actions: ActionForEmpty[];
    boundary?: Boundary | null;
    cw20?: Cw20Coin | null;
    interval: Interval;
    queries?: CroncatQuery[] | null;
    stop_on_fail: boolean;
    transforms?: Transform[] | null;
}
export interface ActionForEmpty {
    gas_limit?: number | null;
    msg: CosmosMsgForEmpty;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
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
export interface BoundaryHeight {
    end?: Uint64 | null;
    start?: Uint64 | null;
}
export interface BoundaryTime {
    end?: Timestamp | null;
    start?: Timestamp | null;
}
export interface Cw20Coin {
    address: string;
    amount: Uint128;
}
export interface CroncatQuery {
    check_result: boolean;
    contract_addr: string;
    msg: Binary;
}
export interface Transform {
    action_idx: number;
    action_path: PathToValue;
    query_idx: number;
    query_response_path: PathToValue;
}
export interface TasksRemoveTaskByManager {
    task_hash: number[];
}
export interface TasksRescheduleTask {
    task_hash: number[];
}
export type QueryMsg =
    | {
          config: {};
      }
    | {
          paused: {};
      }
    | {
          tasks_total: {};
      }
    | {
          current_task_info: {};
      }
    | {
          current_task: {};
      }
    | {
          task: {
              task_hash: string;
          };
      }
    | {
          tasks: {
              from_index?: number | null;
              limit?: number | null;
          };
      }
    | {
          evented_ids: {
              from_index?: number | null;
              limit?: number | null;
          };
      }
    | {
          evented_hashes: {
              from_index?: number | null;
              id?: number | null;
              limit?: number | null;
          };
      }
    | {
          evented_tasks: {
              from_index?: number | null;
              limit?: number | null;
              start?: number | null;
          };
      }
    | {
          tasks_by_owner: {
              from_index?: number | null;
              limit?: number | null;
              owner_addr: string;
          };
      }
    | {
          task_hash: {
              task: Task;
          };
      }
    | {
          slot_hashes: {
              slot?: number | null;
          };
      }
    | {
          slot_ids: {
              from_index?: number | null;
              limit?: number | null;
          };
      }
    | {
          slot_tasks_total: {
              offset?: number | null;
          };
      };
export interface Task {
    actions: ActionForEmpty[];
    amount_for_one_task: AmountForOneTask;
    boundary: Boundary;
    interval: Interval;
    owner_addr: Addr;
    queries: CroncatQuery[];
    stop_on_fail: boolean;
    transforms: Transform[];
    version: string;
}
export interface AmountForOneTask {
    agent_fee: number;
    coin: [Coin | null, Coin | null];
    cw20?: Cw20CoinVerified | null;
    gas: number;
    gas_price: GasPrice;
    treasury_fee: number;
}
export interface Cw20CoinVerified {
    address: Addr;
    amount: Uint128;
}
export interface GasPrice {
    denominator: number;
    gas_adjustment_numerator: number;
    numerator: number;
}
export interface Config {
    chain_name: string;
    croncat_agents_key: [string, [number, number]];
    croncat_factory_addr: Addr;
    croncat_manager_key: [string, [number, number]];
    gas_action_fee: number;
    gas_base_fee: number;
    gas_limit: number;
    gas_query_fee: number;
    owner_addr: Addr;
    pause_admin: Addr;
    slot_granularity_time: number;
    version: string;
}
export interface TaskResponse {
    task?: TaskInfo | null;
}
export interface TaskInfo {
    actions: ActionForEmpty[];
    amount_for_one_task: AmountForOneTask;
    boundary: Boundary;
    interval: Interval;
    owner_addr: Addr;
    queries?: CroncatQuery[] | null;
    stop_on_fail: boolean;
    task_hash: string;
    transforms: Transform[];
    version: string;
}
export interface CurrentTaskInfoResponse {
    last_created_task: Timestamp;
    total: Uint64;
}
export type ArrayOfString = string[];
export type ArrayOfUint64 = number[];
export type ArrayOfTaskInfo = TaskInfo[];
export type Boolean = boolean;
export interface SlotHashesResponse {
    block_id: number;
    block_task_hash: string[];
    time_id: number;
    time_task_hash: string[];
}
export interface SlotIdsResponse {
    block_ids: number[];
    time_ids: number[];
}
export interface SlotTasksTotalResponse {
    block_tasks: number;
    cron_tasks: number;
    evented_tasks: number;
}
export type String = string;
