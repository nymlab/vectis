import assert from "assert";
import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import CosmWasmClient, { downloadContracts } from "../clients/cosmwasm";
import { areContractsDownloaded, getContract, writeToFile } from "../utils/fs";
import {
    hubUploadReportPath,
    remoteUploadReportPath,
    codePaths,
    hostChainName,
    remoteChainName,
} from "../utils/constants";

(async function uploadCode() {
    console.log("Uploading to ", hostChainName, "and ", remoteChainName);
    if (!areContractsDownloaded()) await downloadContracts();

    const daoClient = await CosmWasmClient.connectHostWithAccount("admin");
    const uploadHostRes = await daoClient.uploadHostContracts();
    writeToFile(daoUploadReportPath, JSON.stringify(uploadHostRes, null, 2));
    verifyUpload(uploadHostRes);

    const remoteClient = await CosmWasmClient.connectRemoteWithAccount("admin");
    const uploadRemoteRes = await remoteClient.uploadRemoteContracts();
    writeToFile(remoteUploadReportPath, JSON.stringify(uploadRemoteRes, null, 2));
    verifyUpload(uploadRemoteRes);
})();

async function verifyUpload(contracts: { [key: string]: any }) {
    for (const [key, value] of Object.entries(contracts)) {
        if (key != "dao") {
            const codepath = codePaths[`${key}CodePath`];
            const code = getContract(codepath);
            assert.strictEqual(value.originalChecksum, toHex(sha256(code)));
            assert.strictEqual(value.compressedSize < code.length * 0.5, true);
            assert.strictEqual(value.codeId >= 1, true);
        }
    }
}
