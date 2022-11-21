import assert from "assert";
import { FactoryClient, GovecClient, DaoClient, RelayerClient, CWClient } from "../clients";
import { marketingDescription, marketingProject } from "../clients/govec";
import RemoteTunnelClient from "../clients/remote-tunnel";

import { delay } from "../utils/promises";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import { hostChain, hostChainName, remoteChain, remoteChainName, uploadReportPath } from "../utils/constants";

import type { DaoTunnelT, GovecT, ProxyT } from "../interfaces";
import type { VectisDaoContractsAddrs } from "../interfaces/contracts";

// See README.md for deployment info
//
(async function deploy() {
    const { host, remote } = await import(uploadReportPath);
    const { factoryRes, proxyRes, daoTunnelRes, multisigRes, govecRes } = host;
    const { remoteTunnel, remoteMultisig, remoteProxy, remoteFactory } = remote;

    const relayerClient = new RelayerClient();
    const connection = await relayerClient.connect();
    const channels = await relayerClient.loadChannels();
    const { transfer: channelTransfer } = channels.transfer
        ? channels
        : await relayerClient.createChannel("transfer", "transfer", "ics20-1");

    console.log(connection);
    console.log("ibc transfer channel: ", channelTransfer);

    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const adminRemoteClient = await CWClient.connectRemoteWithAccount("admin");

    const initial_balances: GovecT.Cw20Coin[] = [
        {
            address: adminHostClient.sender,
            amount: "2",
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
    await govecClient.updateConfigAddr({
        newAddr: { staking: daoClient.stakingAddr },
    });
    await govecClient.send({
        amount: "2",
        contract: daoClient.stakingAddr,
        msg: toCosmosMsg({ stake: {} }),
    });
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
    const daoTunnelInstMsg: DaoTunnelT.InstantiateMsg = {
        denom: hostChain.feeToken,
        govec_minter: govecAddr,
        init_ibc_transfer_mods: { endpoints: [[connection.hostConnection, channelTransfer?.src.channelId as string]] },
    };

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

    // Instantiate Remote Tunnel
    const remoteTunnelClient = await RemoteTunnelClient.instantiate(adminRemoteClient, remoteTunnel.codeId, {
        dao_config: {
            addr: daoClient.daoAddr,
            dao_tunnel_port_id: `wasm.${daoTunnelAddr}`,
            connection_id: relayerClient.connections.remoteConnection,
        },
        chain_config: {
            denom: remoteChain.feeToken,
        },
        init_ibc_transfer_mod: {
            endpoints: [[connection.remoteConnection, channelTransfer?.dest.channelId as string]],
        },
    });

    const remoteTunnelAddr = remoteTunnelClient.contractAddress;

    // Admin propose and execute add connection to dao tunnel
    const daoTunnelApproveControllerMsg = daoClient.addApprovedControllerMsg(
        daoTunnelAddr,
        relayerClient.connections.hostConnection,
        `wasm.${remoteTunnelAddr}`
    );

    // Allow connection to remote tunnel
    await daoClient.createProposal("Allow connection in DAO Tunnel", "Allow connection in DAO Tunnel", [
        daoTunnelApproveControllerMsg,
    ]);

    const approveControllerProposalId = 3;
    await delay(10000);

    await daoClient.voteProposal(approveControllerProposalId, "yes");
    await delay(10000);
    await daoClient.executeProposal(approveControllerProposalId);
    await delay(10000);

    // Instantiate Factory in remote chain

    // Create channel
    const { wasm: channelWasm } = await relayerClient.createChannel(
        `wasm.${daoTunnelAddr}`,
        `wasm.${remoteTunnelAddr}`,
        "vectis-v1"
    );

    console.log("ibc wasm channel: ", channelWasm);

    // Admin propose and execute dao deploy factory remote
    const createRemoteFactoryMsg: DaoTunnelT.ExecuteMsg = {
        dispatch_action_on_remote_tunnel: {
            job_id: 1,
            msg: {
                instantiate_factory: {
                    code_id: remoteFactory.codeId,
                    msg: FactoryClient.createFactoryInstMsg(remoteChainName, remoteProxy.codeId, remoteMultisig.codeId),
                },
            },
            channel_id: channelWasm?.src.channelId as string,
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
    await delay(10000);

    const { remote_factory: remoteFactoryAddr } = await remoteTunnelClient.chainConfig();
    console.log("\nRemote Factory Addr: ", remoteFactoryAddr);

    // Update marketing address on Govec
    let res = await govecClient.updateMarketing({
        marketing: daoClient.daoAddr,
    });
    console.log("\n\nUpdated Marketing Address on Govec\n", JSON.stringify(res));

    // Update Proposal address
    res = await govecClient.updateConfigAddr({
        newAddr: { proposal: daoClient.proposalAddr },
    });
    console.log("\n\nUpdated Proposal Address on Govec\n", JSON.stringify(res));

    // Add factory address to Govec
    res = await govecClient.updateConfigAddr({
        newAddr: { factory: factoryAddr },
    });
    console.log("\n\nUpdated Factory Address on Govec\n", JSON.stringify(res));

    // Add dao_tunnel address to Govec
    res = await govecClient.updateConfigAddr({
        newAddr: { dao_tunnel: daoTunnelAddr },
    });
    console.log("\n\nUpdated DaoTunnel Address on Govec\n", JSON.stringify(res));

    // Update DAO address on Govec
    res = await govecClient.updateConfigAddr({
        newAddr: { dao: daoClient.daoAddr },
    });
    console.log("\n\nUpdated Dao Address on Govec\n", JSON.stringify(res));

    res = await adminHostClient.updateAdmin(adminHostClient.sender, govecAddr, daoClient.daoAddr, "auto");
    console.log("\n\nUpdated Govec Contract Admin to DAO\n", JSON.stringify(res));

    res = await adminHostClient.execute(
        adminHostClient.sender,
        daoClient.stakingAddr,
        { unstake: { amount: "2" } },
        "auto"
    );
    console.log("\n\nAdmin unstakes \n", JSON.stringify(res));

    res = await adminHostClient.execute(adminHostClient.sender, govecAddr, { burn: {} }, "auto");
    console.log("\n\nAdmin burns the one govec\n", JSON.stringify(res));

    const contracts = {
        remoteFactoryAddr: remoteFactoryAddr as string,
        remoteTunnelAddr,
        daoTunnelAddr,
        govecAddr,
        factoryAddr,
        daoAddr: daoClient.daoAddr,
        stakingAddr: daoClient.stakingAddr,
        proposalAddr: daoClient.proposalAddr,
        voteAddr: daoClient.voteAddr,
    };

    await verifyDeploy(contracts);

    writeInCacheFolder("deployInfo.json", JSON.stringify(contracts, null, 2));
})();

async function verifyDeploy(addrs: VectisDaoContractsAddrs) {
    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const govecClient = new GovecClient(adminHostClient, adminHostClient.sender, addrs.govecAddr);
    const factoryClient = new FactoryClient(adminHostClient, adminHostClient.sender, addrs.factoryAddr);
    const daoClient = new DaoClient(adminHostClient, {
        daoAddr: addrs.daoAddr,
        proposalAddr: addrs.proposalAddr,
        stakingAddr: addrs.stakingAddr,
        voteAddr: addrs.voteAddr,
    });

    // Deployer account should not have any govec token.
    const { balance } = await govecClient.balance({
        address: adminHostClient.sender,
    });
    assert.strictEqual(balance, "0");

    // DAO should have not admin
    let contract = await adminHostClient.getContract(addrs.daoAddr);
    assert.strictEqual(contract.admin, undefined);

    // DAO Supply should be 0
    const tokenInfo = await govecClient.tokenInfo();
    assert.strictEqual(tokenInfo.total_supply, "0");

    // DAO should have 4 proposals executed
    const { proposals } = await daoClient.queryProposals();
    assert.strictEqual(proposals.length, 4);

    // Govec, dao_tunnel, host_factory, cw20_stake, cw20_stake_balance_voting, Proposal Contracts should have DAO as admin
    contract = await adminHostClient.getContract(addrs.factoryAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);
    contract = await adminHostClient.getContract(addrs.govecAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);
    contract = await adminHostClient.getContract(addrs.stakingAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);
    contract = await adminHostClient.getContract(addrs.voteAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);
    contract = await adminHostClient.getContract(addrs.proposalAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);
    contract = await adminHostClient.getContract(addrs.daoTunnelAddr);
    assert.strictEqual(contract.admin, addrs.daoAddr);

    // Govec should be set on the factory
    const govecAddr = await factoryClient.govecAddr();
    assert.strictEqual(govecAddr, addrs.govecAddr);

    // Govec should have stakingAddr as the stakingAddr
    const stakingAddr = await govecClient.staking();
    assert.strictEqual(stakingAddr, addrs.stakingAddr);

    // Govec should have daoAddr as the dao
    const daoAddr = await govecClient.dao();
    assert.strictEqual(daoAddr, addrs.daoAddr);

    // Govec should have factory addr and dao_tunnel addr
    const { minters } = await govecClient.minters();
    assert.deepStrictEqual(minters, [addrs.daoTunnelAddr, addrs.factoryAddr]);

    // Govec should have have vectis project and description
    const marketingInfo = await govecClient.marketingInfo();
    assert.strictEqual(marketingInfo.project, marketingProject);
    assert.strictEqual(marketingInfo.description, marketingDescription);
}
