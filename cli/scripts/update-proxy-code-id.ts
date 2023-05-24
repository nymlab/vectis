import { FactoryClient, CWClient, Cw3FlexClient } from "../clients";
import { Account } from "../config/accounts";
import { toCosmosMsg } from "../utils/enconding";
import { hostChainName, hubDeployReportPath, hubUploadReportPath, hostAccounts } from "../utils/constants";

import type { FactoryT, Cw3FlexT } from "../interfaces";

(async function update() {
    const { proxy } = await import(hubUploadReportPath);
    const { VectisCommittee, Factory } = await import(hubDeployReportPath);
    const committee1Client = await CWClient.connectHostWithAccount("committee1");
    const committee1 = hostAccounts["committee1"] as Account;
    const vectisComClient = new Cw3FlexClient(committee1Client, committee1.address, VectisCommittee);
    const factoryUpdateCodeIdMsg: FactoryT.ExecuteMsg = {
        update_code_id: {
            new_code_id: proxy.codeId,
            type: "proxy",
        },
    };
    const updateMsg: Cw3FlexT.CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: Factory,
                funds: [],
                msg: toCosmosMsg(factoryUpdateCodeIdMsg),
            },
        },
    };

    const propResult = await vectisComClient.propose(
        {
            description: "Update Proxy CodeId",
            latest: undefined,
            msgs: [updateMsg],
            title: "Update Proxy CodeId",
        },
        "auto"
    );
    console.log("propResult: ", JSON.stringify(propResult));
    const proposals = await vectisComClient.listProposals({});
    const prop = proposals.proposals.pop();
    const propId = prop!.id;
    let execute = await vectisComClient.execute({ proposalId: propId });
    console.log("execute: ", JSON.stringify(execute));
})();
