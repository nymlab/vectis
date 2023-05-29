import assert from "assert";
import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import CosmWasmClient, { downloadContracts } from "../clients/cosmwasm";
import { areContractsDownloaded, getContract, writeToFile } from "../utils/fs";
import { hubUploadReportPath, coreCodePaths, pluginCodePaths, hostChainName } from "../utils/constants";

(async function uploadCode() {
    if (!areContractsDownloaded()) await downloadContracts();
    console.log("Uploading to ", hostChainName);
    const hostClient = await CosmWasmClient.connectHostWithAccount("admin");
    const uploadHostRes = await hostClient.uploadHubContracts();
    console.log("Upload Res, ", uploadHostRes);
    writeToFile(hubUploadReportPath, JSON.stringify(uploadHostRes, null, 2));
    verifyUpload(uploadHostRes);
})();

async function verifyUpload(contracts: { [key: string]: any }) {
    for (const [key, value] of Object.entries(contracts)) {
        if (key != "plugins") {
            const codepath = coreCodePaths[`${key}CodePath`];
            const code = getContract(codepath);
            assert.strictEqual(value.originalChecksum, toHex(sha256(code)));
            assert.strictEqual(value.compressedSize < code.length * 0.5, true);
            assert.strictEqual(value.codeId >= 1, true);
        } else {
            for (const [key, value] of Object.entries(contracts.plugins)) {
                const codepath = pluginCodePaths[`${key}CodePath`];
                const code = getContract(codepath);
                assert.strictEqual(value.originalChecksum, toHex(sha256(code)));
                assert.strictEqual(value.compressedSize < code.length * 0.5, true);
                assert.strictEqual(value.codeId >= 1, true);
            }
        }
    }
}
