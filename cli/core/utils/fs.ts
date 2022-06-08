import fs from "fs";
import path from "path";
import { cachePath } from "./constants";

export function getContract(path: string): Uint8Array {
    return fs.readFileSync(path);
}

export function writeInCacheFolder(fileName: string, content: string, encoding: BufferEncoding = "utf8"): void {
    if (!fs.existsSync(cachePath)) fs.mkdirSync(cachePath);
    fs.writeFileSync(path.join(cachePath, fileName), content, { encoding });
}
