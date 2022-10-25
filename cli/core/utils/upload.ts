import CosmWasmClient, { downloadContracts } from "../clients/cosmwasm";
import { areContractsDownloaded } from "./fs";

import type { Chains } from "../config/chains";

export async function uploadCode(hostChain: Chains, remoteChain: Chains) {
    if (!areContractsDownloaded()) await downloadContracts();

    const daoClient = await CosmWasmClient.connectWithAccount(hostChain, "admin");
    const uploadHostRes = await daoClient.uploadHostContracts(hostChain);

    const remoteClient = await CosmWasmClient.connectWithAccount(remoteChain, "admin");
    const uploadRemoteRes = await remoteClient.uploadRemoteContracts();

    return { host: { ...uploadHostRes }, remote: { ...uploadRemoteRes } };
}
