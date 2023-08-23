import { Chains } from "./config/chains";
import path from "path";
import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import CosmWasmClient from "./clients/cosmwasm";
import { Logger } from "tslog";
import { writeToFile, getWasmFileNames, getUploadInfoPath } from "./utils/fs";
import { wasmArtifactsPath } from "./config/fs";
import { OptionValues } from "commander";

export async function uploadAction(network: Chains, opts: OptionValues) {
    const logger = new Logger();
    if (opts.vectis && opts.contracts) {
        logger.fatal("Please choose to upload one of `contracts` or `Vectis`");
    }
    const client = await CosmWasmClient.connectHostWithAccount("admin", network);
    const artifactContracts = getWasmFileNames("../artifacts");
    let contractsToUplaod: string[] = [];
    if (!opts.contracts.length) {
        logger.info("Uploading all: ", artifactContracts);
        contractsToUplaod = artifactContracts;
    } else {
        opts.contracts.map((c: string) => {
            if (!artifactContracts.includes(c)) {
                throw new Error(`Contract ${c} wasm file not found in ${wasmArtifactsPath}`);
            } else {
                contractsToUplaod.push(c);
            }
        });
    }

    const uploadPath = getUploadInfoPath(network);
    let uploadedContracts: Record<string, UploadResult>;
    let newUploadedContracts: Record<string, UploadResult> = {};
    try {
        uploadedContracts = await import(uploadPath);
    } catch (_) {
        uploadedContracts = {};
    }

    // there is no setter for `uploadedContracts`
    Object.assign(newUploadedContracts, uploadedContracts);

    for (let c of contractsToUplaod) {
        await new Promise(async (resolve) => {
            const contractPath = path.join(wasmArtifactsPath, `/${c}`);
            logger.info("Uploading: ", contractPath);

            // substring ".wasm"
            const contractName = c.substring(0, c.length - 5);
            let res = await client.uploadContract(contractPath);
            newUploadedContracts[contractName] = res;
            resolve(newUploadedContracts);
        });
    }

    writeToFile(uploadPath, JSON.stringify(newUploadedContracts, null, 2));
    console.log("Wrote results to file: ", uploadPath);
}
