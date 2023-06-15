import fs from "fs";
import { MsgBroadcasterWithPk, MsgStoreCode, PrivateKey } from "@injectivelabs/sdk-ts";
import { getNetworkEndpoints, Network } from "@injectivelabs/networks";
import { coreCodePaths } from "../../utils/constants";
import * as injectiveAccounts from "../../config/accounts/injective";
import { writeToFile } from "../../utils/fs";
import path from "path";

(async function uploadCode() {
    const network = process.env.HOST_CHAIN;
    console.log("Uploading to ", network);
    const { admin } = injectiveAccounts[network as keyof typeof injectiveAccounts];
    const privateKey = PrivateKey.fromMnemonic(admin.mnemonic!);
    const endpoints = getNetworkEndpoints(Network.TestnetK8s);

    const codesId = {} as Record<string, number>;

    for await (const [key, value] of Object.entries(coreCodePaths)) {
        try {
            const contract = fs.readFileSync(value);
            const msg = MsgStoreCode.fromJSON({
                sender: admin.address!,
                wasmBytes: contract,
            });

            const txHash = await new MsgBroadcasterWithPk({
                privateKey,
                network: Network.Testnet,
                endpoints: endpoints,
                simulateTx: true,
            }).broadcast({
                msgs: msg,
                injectiveAddress: admin.address!,
            });

            const [{ events }] = JSON.parse(txHash.rawLog);
            const { attributes } = events.find((e: any) => e.type === "cosmwasm.wasm.v1.EventCodeStored");
            const { value: codeId } = attributes.find((a: any) => a.key === "code_id");
            const name = key.replace("Path", "Id");
            codesId[name] = Number(codeId);
            console.log("Uploaded: ", name, codeId);
        } catch (err) {
            console.log("Upload failed: ", key, "errr: ", err);
        }
    }
    let deployPath = path.join(__dirname, "../../deploy", `${process.env.HOST_CHAIN}-uploadInfo-test.json`);
    writeToFile(deployPath, JSON.stringify(codesId, null, 2));
})();
