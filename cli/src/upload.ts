import { SupportChains, Chain } from "./config/chains";
import { VectisContracts } from "./config/contracts";
import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import CosmWasmClient from "./clients/cosmwasm";
import { Logger } from "tslog";
import * as chainConfigs from "./config/chains";
import _ from "lodash";
import { hubUploadReportPath, coreCodePaths } from "./utils/constants";
import { writeToFile } from "./utils/fs";

export async function uploadAction(network: string, contracts: string[]) {
    const logger = new Logger();
    if (!(network in SupportChains)) {
        logger.fatal(new Error("Network not supported"));
    }

    const client = await CosmWasmClient.connectHostWithAccount("admin", network);
    const chain = chainConfigs[network as keyof typeof chainConfigs] as Chain;

    if (!contracts.length) {
        logger.info("Uploading all");
        const uploadHostRes = await client.uploadHubContracts(chain);
        writeToFile(hubUploadReportPath(chain), JSON.stringify(uploadHostRes, null, 2));
    } else {
        _.map(contracts, async (c) => {
            if (!(c in VectisContracts)) logger.error(new Error("Contract not supported"));
            let codePath = coreCodePaths[`${c}CodePath`];
            const result = await client.uploadContract(codePath);
            console.log("upload results for: ", c);
            console.log(JSON.stringify(result));
            try {
                const uplaodRes = await import(hubUploadReportPath(chain));
                uplaodRes[c] = result;
                writeToFile(hubUploadReportPath(chain), JSON.stringify(uplaodRes, null, 2));
            } catch (_) {
                const report: { [key: string]: UploadResult } = {};
                report[c] = result;
                writeToFile(hubUploadReportPath(chain), JSON.stringify(report, null, 2));
            }
        });
    }
}
