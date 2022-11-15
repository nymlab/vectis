import fs from "fs";
import path from "path";
import axios from "axios";
import { cachePath, downloadContractPath, contractsFileNames, configPath } from "./constants";

export function getContract(path: string): Uint8Array {
    return fs.readFileSync(path);
}

export function writeInCacheFolder(fileName: string, content: string, encoding: BufferEncoding = "utf8"): void {
    if (!fs.existsSync(cachePath)) fs.mkdirSync(cachePath);
    fs.writeFileSync(path.join(cachePath, fileName), content, { encoding });
}

export function writeRelayerConfig(data: unknown, fileName: string): void {
    fs.writeFileSync(path.join(configPath, "/relayer", fileName), JSON.stringify(data, null, 2));
}

export function loadIbcInfo(): Record<string, any> | null {
    const ibcInfoPath = path.join(cachePath, "ibcInfo.json");
    if (!fs.existsSync(ibcInfoPath)) return null;
    const ibcInfo = fs.readFileSync(path.join(cachePath, "ibcInfo.json")).toString();
    return ibcInfo ? JSON.parse(ibcInfo) : null;
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

export async function downloadContract(url: string, fileName: string): Promise<void> {
    if (!fs.existsSync(downloadContractPath)) fs.mkdirSync(downloadContractPath, { recursive: true });
    await downloadFile(url, `contracts/${fileName}`);
}

export function areContractsDownloaded(): boolean {
    if (!fs.existsSync(downloadContractPath)) return false;
    const downloadContractPathFiles = fs.readdirSync(downloadContractPath);
    return Object.values(contractsFileNames).every((fileName) => downloadContractPathFiles.includes(fileName));
}
