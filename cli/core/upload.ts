import CosmWasmClient, { downloadContracts } from "./clients/cosmwasm";
import { areContractsDownloaded, writeInCacheFolder } from "./utils/fs";

import type { Chains } from "./config/chains";

async function uploadCode() {
    const [hostChain, remoteChain] = process.argv.slice(2) as Chains[];
    if (!areContractsDownloaded()) await downloadContracts();

    const daoClient = await CosmWasmClient.connectWithAccount(hostChain, "admin");
    const uploadHostRes = await daoClient.uploadHostContracts(hostChain);

    const remoteClient = await CosmWasmClient.connectWithAccount(remoteChain, "admin");
    const uploadRemoteRes = await remoteClient.uploadRemoteContracts();

    writeInCacheFolder(
        "uploadInfo.json",
        JSON.stringify({ host: { ...uploadHostRes }, remote: { ...uploadRemoteRes } }, null, 2)
    );
}

uploadCode();
