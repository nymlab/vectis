import { createSigningClient, downloadContracts, uploadContracts } from "./services/cosmwasm";
import { addrPrefix, adminMnemonic } from "./utils/constants";
import { areContractsDownloaded, writeInCacheFolder } from "./utils/fs";

async function uploadCode() {
    if (!areContractsDownloaded()) await downloadContracts();

    const adminClient = await createSigningClient(adminMnemonic, addrPrefix);
    const uploadRes = await uploadContracts(adminClient);
    writeInCacheFolder("uploadInfo.json", JSON.stringify(uploadRes, null, 2));
}

uploadCode();
