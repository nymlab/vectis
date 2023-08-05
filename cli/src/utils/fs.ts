import fs from "fs";
import path from "path";
import axios from "axios";
import { cachePath, wasmArtifactsPath, contractSchemaPath, accountsPath, deployResultsPath } from "../config/fs";

export function getContract(path: string): Uint8Array {
    return fs.readFileSync(path);
}

export function getWasmFileNames(path: string): string[] {
    return fs.readdirSync(path).filter((v) => v.includes(".wasm"));
}

export function writeToFile(fullPath: string, content: string, encoding: BufferEncoding = "utf8"): void {
    const dir = path.dirname(fullPath);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(fullPath, content, { encoding });
}

export async function downloadFile(url: string, fileName: string): Promise<void> {
    const file = fs.createWriteStream(path.join(cachePath, fileName));

    const { data } = await axios.get(url, { responseType: "stream" });
    data.pipe(file);

    return new Promise((resolve, reject) => {
        file.on("finish", resolve);
        file.on("error", reject);
    });
}

export async function downloadContractWasm(url: string, fileName: string): Promise<void> {
    if (!fs.existsSync(wasmArtifactsPath)) fs.mkdirSync(wasmArtifactsPath, { recursive: true });
    await downloadFile(url, `${fileName}`);
}

export async function downloadTypeSchema(url: string, contractName: string, fileName: string): Promise<void> {
    let dir = path.join(contractSchemaPath, contractName);
    if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
    await downloadFile(url, `schemas/${contractName}/${fileName}`);
}

export const getAccountsPath = (network: string) => path.join(accountsPath, `/${network}.json`);
export const getUploadInfoPath = (network: string, vectis?: Boolean) =>
    path.join(deployResultsPath, vectis ? `/${network}-vectis-uploadInfo.json` : `/${network}-uploadInfo.json`);
export const getDeployPath = (network: string) => path.join(deployResultsPath, `/${network}-vectis-deployInfo.json`);
