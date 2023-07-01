import { Chains } from "./config/chains";
import { VectisContractsUploadResult } from "./config/contracts";
import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import { Logger } from "tslog";

export function uploadAction(network: string, contracts: string[]) {
  const logger = new Logger();

  if (!Chains.find((e) => e == network)) {
    logger.fatal(new Error("Network not supported"));
  }

  const toUpload = [];
  if (!contracts.length) {
    logger.info("Uploading all");
  } else {
    logger.info("Uploading: ", contracts);
  }
}
