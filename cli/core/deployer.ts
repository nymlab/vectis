import FactoryClient from "@vectis/core/clients/factory";
import GovecClient from "@vectis/core/clients/govec";
import DaoClient from "@vectis/core/clients/dao";
import { toCosmosMsg } from "@vectis/core/utils/enconding";
import { uploadReportPath } from "./utils/constants";
import * as ACCOUNTS from "./config/accounts";
import * as CHAINS from "./config/chains";

import RelayerClient from "@vectis/core/clients/relayer";
import CWClient from "@vectis/core/clients/cosmwasm";

import { DaoTunnelT, GovecT, ProxyT } from "@vectis/types";
import { CosmosMsg_for_Empty } from "types/build/contracts/ProxyContract";
import type { Chains } from "./config/chains";
import { writeInCacheFolder } from "./utils/fs";

// Deployment
// The deployment of the DAO on the host chain has the following steps:
//
// 1. Upload all required contracts (in ./upload.ts) to host chain + remote chains
// 		- Host: Factory, Govec, Proxy, Ibc-host
// 		- Remote: Factory-remote, Proxy-remote, Ibc-remote
// 2. Instantiate Govec contract (with admin having initial balance for proposing for DAO to deploy Factory)
// 3. Instantiate dao-core contract (which will instantiate proposal(s) and vote contracts)
//    note: vote contracts also instantiates a new staking contract
// 4. Admin propose and execute on DAO to deploy factory and ibc-host contracts
// 5. Admin updates Govec staking address to the staking contract in step 3.
// 5. Admin updates Govec minter address to the factory contract addr and ibc-host addr in step 4.
// 6. Admin updates Govec contract DAO_ADDR as DAO
// 7. Admin updates Govec contract admin as DAO (for future upgrades)
//
//    (Somehow DAO gets ICA on other chains?)
//
// 8. Admin propose and execute on DAO to deploy factory-remote and Ibc-remote contracts with DAO ICA
// 9. Create channels between Ibc-host<> Ibc-remote_1; Ibc-host <> Ibc-remote_2; with known connection ids?
// 10. Admin propose and execute to add <connection-id, portid> to ibc-host for each channel in step 9
// 11. Admin unstakes and burn its govec (exits system)
//

export async function deploy(): Promise<void> {
    const { host, remote } = await import(uploadReportPath);
    const { factoryRes, proxyRes, daoTunnelRes, multisigRes, govecRes } = host;
    const { remoteTunnel, remoteProxy, remoteFactory } = remote;

    const [hostChainName, remoteChainName] = process.argv.slice(2) as Chains[];
    const hostChain = CHAINS[hostChainName];
    const hostAccounts = ACCOUNTS[hostChainName.split("_")[0] as keyof typeof ACCOUNTS];
    const remoteAccounts = ACCOUNTS[remoteChainName.split("_")[0] as keyof typeof ACCOUNTS];

    const relayerClient = new RelayerClient(hostChainName, remoteChainName);
    await relayerClient.createConnection();

    const {
        admin: { address: hostAdminAddr },
    } = hostAccounts as ACCOUNTS.Account;

    const {
        admin: { address: remoteAdminAddr },
    } = remoteAccounts as ACCOUNTS.Account;

    const adminClient = await CWClient.connectWithAccount(hostChainName, "admin");

    const initial_balances: GovecT.Cw20Coin[] = [
        {
            address: hostAdminAddr,
            amount: "1",
        },
    ];

    // Instantiate Govec
    const govecClient = await GovecClient.instantiate(adminClient, govecRes.codeId, {
        initial_balances,
        marketing: GovecClient.createVectisMarketingInfo(hostAdminAddr),
    });
    const govecAddr = govecClient.contractAddress;
    console.log("Instantiated Govec at: ", govecAddr);

    // Instantiate DAO
    const daoClient = await DaoClient.instantiate(adminClient, govecAddr);

    // Admin stake initial balance to prepare for proposal
    await govecClient.updateConfigAddr({ newAddr: { staking: daoClient.stakingAddr } });
    await govecClient.send({ amount: "1", contract: daoClient.stakingAddr, msg: toCosmosMsg({ stake: {} }) });

    // Admin propose and execute dao deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(
        hostChainName,
        proxyRes.codeId,
        multisigRes.codeId,
        govecAddr
    );

    const deployFactoryMsg: CosmosMsg_for_Empty = {
        wasm: {
            instantiate: {
                admin: daoClient.daoAddr,
                code_id: factoryRes.codeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };

    // Propose instantiate factory
    await daoClient.createProposal("Deploy Vectis Factory", "Deploy Vectis Factory", [deployFactoryMsg]);
    const factoryProposalId = 0;

    // Vote and Execute to deploy Factory
    await daoClient.voteProposal(factoryProposalId, "yes");
    const factoryInstRes = await daoClient.executeProposal(factoryProposalId);
    const factoryAddr = CWClient.getContractAddrFromResult(factoryInstRes, "Vectis Factory instantiated");

    // Admin propose and execute dao deploy dao tunnel
    const daoTunnelInstMsg: DaoTunnelT.InstantiateMsg = { govec_minter: govecAddr };

    const deployDaoTunnelMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            instantiate: {
                admin: daoClient.daoAddr,
                code_id: daoTunnelRes.codeId,
                funds: [],
                label: "Vectis DAO Tunnel",
                msg: toCosmosMsg(daoTunnelInstMsg),
            },
        },
    };

    // Propose instantiate dao tunnel
    await daoClient.createProposal("Deploy Vectis DAO Tunnel", "Deploy Vectis DAO Tunnel", [deployDaoTunnelMsg]);
    const daoTunnelProposalId = 1;

    // Vote and Execute to deploy dao tunnel
    await daoClient.voteProposal(daoTunnelProposalId, "yes");
    const daoTunnelInstRes = await daoClient.executeProposal(daoTunnelProposalId);

    const daoTunnelAddr = CWClient.getContractAddrFromResult(daoTunnelInstRes, "Vectis DAO-Tunnel instantiated");

    // Update marketing address on Govec
    let res = await govecClient.updateMarketing({ marketing: daoClient.daoAddr });
    console.log("\n\nUpdated Marketing Address on Govec\n", JSON.stringify(res));

    // Add factory address to Govec
    res = await govecClient.updateConfigAddr({ newAddr: { factory: factoryAddr } });
    console.log("\n\nUpdated Factory Address on Govec\n", JSON.stringify(res));

    // Add dao_tunnel address to Govec
    res = await govecClient.updateConfigAddr({ newAddr: { dao_tunnel: daoTunnelAddr } });
    console.log("\n\nUpdated DaoTunnel Address on Govec\n", JSON.stringify(res));

    // Update DAO address on Govec
    res = await govecClient.updateConfigAddr({ newAddr: { dao: daoClient.daoAddr } });
    console.log("\n\nUpdated Dao Address on Govec\n", JSON.stringify(res));

    res = await adminClient.updateAdmin(hostAdminAddr, govecAddr, daoClient.daoAddr, "auto");
    console.log("\n\nUpdated Govec Contract Admin to DAO\n", JSON.stringify(res));

    res = await adminClient.execute(hostAdminAddr, daoClient.stakingAddr, { unstake: { amount: "1" } }, "auto");
    console.log("\n\nAdmin unstakes \n", JSON.stringify(res));

    //// Below is only needed if theres is an unstake period
    // delay(5000);
    // res = await adminClient.execute(adminAddr!, stakingAddr, { claim: {} }, defaultExecuteFee);
    // console.log("\n\nAdmin claim \n", JSON.stringify(res));

    res = await adminClient.execute(hostAdminAddr, govecAddr, { burn: {} }, "auto");
    console.log("\n\nAdmin burns the one govec\n", JSON.stringify(res));

    const vectisContracts = {
        govecAddr,
        factoryAddr,
        daoAddr: daoClient.daoAddr,
        stakingAddr: daoClient.stakingAddr,
        proposalAddr: daoClient.proposalAddr,
        voteAddr: daoClient.voteAddr,
    };

    writeInCacheFolder("deployInfo.json", JSON.stringify(vectisContracts, null, 2));
}
