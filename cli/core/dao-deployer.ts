import { deploy } from "./utils/dao-deploy";
import { writeInCacheFolder } from "./utils/fs";

async function deployContracts() {
    const vectisDaoContractsAddrs = await deploy();
    writeInCacheFolder("deployInfo.json", JSON.stringify(vectisDaoContractsAddrs, null, 2));
}

deployContracts();
