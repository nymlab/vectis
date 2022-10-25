import { Chains } from "@vectis/core/config/chains";
import { writeInCacheFolder } from "@vectis/core/utils/fs";
import { deploy as daoDeploy } from "../deploy/dao";

async function deploy() {
    const [hostChain, remoteChain] = process.argv.slice(2) as Chains[];
    const vectisContracts = await daoDeploy(hostChain, remoteChain);

    writeInCacheFolder("deployInfo.json", JSON.stringify(vectisContracts, null, 2));
}

deploy();
