import type { UploadResult } from "@cosmjs/cosmwasm-stargate";

export interface VectisContractsUploadResult {
  factory: UploadResult;
  proxy: UploadResult;
  pluginReg: UploadResult;
  cw3Fixed: UploadResult;
  cw3Flex: UploadResult;
  cw4Group: UploadResult;
  plugins: { [key: string]: UploadResult };
}

// These are all the contract on the Hub Chain
export interface VectisContractsAddrs {
  PluginCommittee: string;
  PluginCommitteeGroup: string;
  VectisCommittee: string;
  VectisCommitteeGroup: string;
  Factory: string;
  PluginRegistry: string;
}
