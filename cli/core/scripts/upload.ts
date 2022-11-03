import { Chains } from "../config/chains";
import { writeInCacheFolder } from "../utils/fs";
import { uploadCode } from "../utils/upload";

const upload = async () => {
    const [hostChain, remoteChain] = process.argv.slice(2) as Chains[];

    const contracts = await uploadCode(hostChain, remoteChain);

    writeInCacheFolder("uploadInfo.json", JSON.stringify(contracts, null, 2));
};

upload();
