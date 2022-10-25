import { deploy } from "@vectis/core/deploy/dao";
import { deployReportPath } from "@vectis/core/utils/constants";
import { writeInCacheFolder } from "@vectis/core/utils/fs";
import { uploadCode } from "@vectis/core/utils/upload";

const setup = async (): Promise<void> => {
    if (process.env.UPLOAD === "false") return (global.contracts = await import(deployReportPath));
    console.info("\nUploading Contracts");
    const codes = await uploadCode("juno_localnet", "wasm_localnet");
    writeInCacheFolder("uploadInfo.json", JSON.stringify(codes, null, 2));

    console.info("\nDeploying Contracts");
    const contracts = await deploy("juno_localnet", "wasm_localnet");
    writeInCacheFolder("deployInfo.json", JSON.stringify(contracts, null, 2));
    global.contracts = contracts;
};

export default setup;
