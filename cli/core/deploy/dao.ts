import FactoryClient from "@vectis/core/clients/factory";
import GovecClient from "@vectis/core/clients/govec";
import DaoClient from "@vectis/core/clients/dao";
import { toCosmosMsg } from "@vectis/core/utils/enconding";
import { uploadReportPath } from "../utils/constants";

import RelayerClient from "@vectis/core/clients/relayer";
import CWClient from "@vectis/core/clients/cosmwasm";

import { DaoTunnelT, GovecT, ProxyT } from "@vectis/types";
import type { Chains } from "../config/chains";
import { writeInCacheFolder } from "../utils/fs";
import { delay } from "../utils/promises";
import { VectisDaoContractsAddrs } from "../interfaces/dao";

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

export async function deploy(hostName?: Chains, remoteName?: Chains): Promise<VectisDaoContractsAddrs> {
    console.log(hostName, remoteName);
    const { host, remote } = await import(uploadReportPath);
    const { factoryRes, proxyRes, daoTunnelRes, multisigRes, govecRes } = host;
    const { remoteTunnel, remoteMultisig, remoteProxy, remoteFactory } = remote;

    const hostChainName = hostName ? hostName : (process.argv.slice(2)[0] as Chains);
    const remoteChainName = remoteName ? remoteName : (process.argv.slice(2)[1] as Chains);

    const relayerClient = new RelayerClient(hostChainName, remoteChainName);
    const connection = await relayerClient.createConnection();
    console.log(connection);

    const adminHostClient = await CWClient.connectWithAccount(hostChainName, "admin");
    const adminRemoteClient = await CWClient.connectWithAccount(remoteChainName, "admin");

    const initial_balances: GovecT.Cw20Coin[] = [
        {
            address: adminHostClient.sender,
            amount: "1",
        },
    ];

    // Instantiate Govec
    const govecClient = await GovecClient.instantiate(adminHostClient, govecRes.codeId, {
        initial_balances,
        marketing: GovecClient.createVectisMarketingInfo(adminHostClient.sender),
    });
    const govecAddr = govecClient.contractAddress;
    console.log("Instantiated Govec at: ", govecAddr);

    // Instantiate DAO
    const daoClient = await DaoClient.instantiate(adminHostClient, govecAddr);

    // Admin stake initial balance to prepare for proposal
    await govecClient.updateConfigAddr({ newAddr: { staking: daoClient.stakingAddr } });
    await govecClient.send({ amount: "1", contract: daoClient.stakingAddr, msg: toCosmosMsg({ stake: {} }) });
    await delay(10000);

    // Admin propose and execute dao deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(
        hostChainName,
        proxyRes.codeId,
        multisigRes.codeId,
        govecAddr
    );

    const deployFactoryMsg = {
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
    const factoryProposalId = 1;
    await delay(10000);

    // Vote and Execute to deploy Factory
    await daoClient.voteProposal(factoryProposalId, "yes");
    await delay(10000);
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
    const daoTunnelProposalId = 2;
    await delay(10000);

    // Vote and Execute to deploy dao tunnel
    await daoClient.voteProposal(daoTunnelProposalId, "yes");
    await delay(10000);
    const daoTunnelInstRes = await daoClient.executeProposal(daoTunnelProposalId);

    const daoTunnelAddr = CWClient.getContractAddrFromResult(daoTunnelInstRes, "Vectis DAO-Tunnel instantiated");

    ///// Remote chain
    // Instantiate Remote Tunnel
    const { contractAddress: remoteTunnelAddr } = await adminRemoteClient.instantiate(
        adminRemoteClient.sender,
        remoteTunnel.codeId,
        { connection_id: relayerClient.connections.remoteConnection, port_id: `wasm.${daoTunnelAddr}` },
        "Vectis Remote tunnel",
        "auto"
    );

    // Admin propose and execute add connection to dao tunnel
    const daoTunnelApproveControllerMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: daoTunnelAddr,
                funds: [],
                msg: toCosmosMsg({
                    add_approved_controller: {
                        connection_id: relayerClient.connections.hostConnection,
                        port_id: `wasm.${remoteTunnelAddr}`,
                    },
                }),
            },
        },
    };

    // Allow connection to remote tunnel
    await daoClient.createProposal("Allow connection in DAO Tunnel", "Allow connection in DAO Tunnel", [
        daoTunnelApproveControllerMsg,
    ]);
    const approveControllerProposalId = 3;
    await delay(10000);

    await daoClient.voteProposal(approveControllerProposalId, "yes");
    await delay(10000);
    await daoClient.executeProposal(approveControllerProposalId);
    await delay(15000);

    // Instantiate Factory in remote chain

    // Create channel
    const channels = await relayerClient.createChannel(`wasm.${daoTunnelAddr}`, `wasm.${remoteTunnelAddr}`);

    console.log(channels);

    // Admin propose and execute dao deploy factory remote
    const createRemoteFactoryMsg: DaoTunnelT.ExecuteMsg = {
        instantiate_remote_factory: {
            code_id: remoteFactory.codeId,
            msg: FactoryClient.createFactoryInstMsg(remoteChainName, remoteProxy.codeId, remoteMultisig.codeId),
            channel_id: channels.hostChannel,
        },
    };

    const createRemoteFactoryProposalMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: daoTunnelAddr,
                funds: [],
                msg: toCosmosMsg(createRemoteFactoryMsg),
            },
        },
    };
    await daoClient.createProposal("Instantiate Factory in Remote Chain", "Instantiate Factory in Remote Chain", [
        createRemoteFactoryProposalMsg,
    ]);
    await delay(10000);
    const instRemoteFactoryProposalId = 4;

    await daoClient.voteProposal(instRemoteFactoryProposalId, "yes");
    await delay(10000);
    await daoClient.executeProposal(instRemoteFactoryProposalId);
    await delay(10000);

    // Relay packets and acknowledge
    await relayerClient.relayAll();

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

    res = await adminHostClient.updateAdmin(adminHostClient.sender, govecAddr, daoClient.daoAddr, "auto");
    console.log("\n\nUpdated Govec Contract Admin to DAO\n", JSON.stringify(res));

    res = await adminHostClient.execute(
        adminHostClient.sender,
        daoClient.stakingAddr,
        { unstake: { amount: "1" } },
        "auto"
    );
    console.log("\n\nAdmin unstakes \n", JSON.stringify(res));

    //// Below is only needed if theres is an unstake period
    // delay(10000);
    // res = await adminHostClient.execute(adminAddr!, stakingAddr, { claim: {} }, defaultExecuteFee);
    // console.log("\n\nAdmin claim \n", JSON.stringify(res));

    res = await adminHostClient.execute(adminHostClient.sender, govecAddr, { burn: {} }, "auto");
    console.log("\n\nAdmin burns the one govec\n", JSON.stringify(res));

    return {
        remoteTunnelAddr,
        daoTunnelAddr,
        govecAddr,
        factoryAddr,
        daoAddr: daoClient.daoAddr,
        stakingAddr: daoClient.stakingAddr,
        proposalAddr: daoClient.proposalAddr,
        voteAddr: daoClient.voteAddr,
    };
}
