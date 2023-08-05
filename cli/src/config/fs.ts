import path from "path";

export const cachePath = path.join(__dirname, "../../.cache");
export const deployResultsPath = path.join(__dirname, "../../deploy");
export const wasmArtifactsPath = path.join(__dirname, "../../../artifacts");
export const contractSchemaPath = path.join(cachePath, "/schemas");
export const accountsPath = path.join(cachePath, "/accounts");
