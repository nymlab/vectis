import { Chains } from "./config/chains";
import path from "path";
import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import CosmWasmClient from "./clients/cosmwasm";
import { Logger } from "tslog";
import { writeToFile, getWasmFileNames, getUploadInfoPath } from "./utils/fs";
import { wasmArtifactsPath, deployResultsPath } from "./config/fs";
import { OptionValues } from "commander";
import { ProxyClient } from "./interfaces";

export async function test(network: Chains, opts: OptionValues) {
    const logger = new Logger();
    if (opts.vectis && opts.contracts) {
        logger.fatal("Please choose to upload one of `contracts` or `Vectis`");
    }
    const client = await CosmWasmClient.connectHostWithAccount("admin", network);
    // read deploy
    // connect factory + proxy
    // factory: createWalletWebAuthn
    // proxy: relayTxFromSelf
}
