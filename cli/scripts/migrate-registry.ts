import assert from "assert";
import { FactoryClient, CWClient, Cw3FlexClient, Cw4GroupClient, PluginRegClient } from "../clients";
import { Account } from "../config/accounts";
import { toCosmosMsg } from "../utils/enconding";
import { writeToFile } from "../utils/fs";
import { hostChainName, hubDeployReportPath, hubUploadReportPath, hostAccounts } from "../utils/constants";

import type { VectisHubChainContractsAddrs } from "../interfaces/contracts";
import {
    vectisCommittee1Weight,
    vectisCommittee2Weight,
    vectisTechCommittee1Weight,
    vectisTechCommittee2Weight,
} from "../clients/cw3flex";
import { VectisActors } from "../utils/constants";

(async function migrate() {
    console.log("migrate registry", hostChainName);
    const { VectisCommittee, PluginRegistry } = await import(hubDeployReportPath);
    const { pluginReg, plugins } = await import(hubUploadReportPath);
    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const committee1 = hostAccounts["committee1"] as Account;
    const committee1Client = await CWClient.connectHostWithAccount("committee1");
    const vectisComClient = new Cw3FlexClient(committee1Client, committee1.address, VectisCommittee);
    let res = await vectisComClient.migrate(PluginRegistry, pluginReg.codeId);
    console.log(JSON.stringify(res));
})();
