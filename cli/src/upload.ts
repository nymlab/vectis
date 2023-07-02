import { SupportChains, Chain } from "./config/chains";
import { VectisContracts } from "./config/contracts";
import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import CosmWasmClient from "./clients/cosmwasm";
import { Logger } from "tslog";
import * as chainConfigs from "./config/chains";
import _ from "lodash";
import { hubUploadReportPath, coreCodePaths, pluginCodePaths } from "./utils/constants";
import { writeToFile } from "./utils/fs";

export async function uploadAction(network: string, contracts: string[]) {
    const logger = new Logger();
    if (!(network in SupportChains)) {
        logger.fatal(new Error("Network not supported"));
    }

    const chain = chainConfigs[network as keyof typeof chainConfigs] as Chain;
    const client = await CosmWasmClient.connectHostWithAccount("admin", chain);

    const toUpload = [];
    if (!contracts.length) {
        logger.info("Uploading all");
        const uploadHostRes = await client.uploadHubContracts(chain);
        writeToFile(hubUploadReportPath, JSON.stringify(uploadHostRes, null, 2));
    } else {
        _.map(contracts, (c) => {
            if (!(c in VectisContracts)) logger.error(new Error("Network not supported"));
        });
    }
}
