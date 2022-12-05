import assert from "assert";
import { FactoryClient, GovecClient, DaoClient, RelayerClient, CWClient } from "../clients";
import { marketingDescription, marketingProject } from "../clients/govec";
import RemoteTunnelClient from "../clients/remote-tunnel";
import { govecGenesisBalances } from "../config/govec_init_balances";

import { Account } from "../config/accounts";
import { delay } from "../utils/promises";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import {
    hostChain,
    hostChainName,
    remoteChain,
    remoteChainName,
    uploadReportPath,
    hostAccounts,
} from "../utils/constants";

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

    console.log("\n ibc transfer channel: ", channelTransfer);
    console.log(connection);

    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const adminRemoteClient = await CWClient.connectRemoteWithAccount("admin");

    // Instantiate Govec
    const govecClient = await GovecClient.instantiate(adminHostClient, govecRes.codeId, {
        initial_balances: govecGenesisBalances,
        marketing: GovecClient.createVectisMarketingInfo(adminHostClient.sender),
        mintAmount: "2",
    });
    const govecAddr = govecClient.contractAddress;
    console.log("Instantiated Govec at: ", govecAddr);

    // Instantiate DAO with Admin to help with deploy and dao-action tests
    // Admin will be removed at the end of the tests
    const daoAdmin = hostAccounts["admin"] as Account;
    const daoClient = await DaoClient.instantiate(adminHostClient, govecAddr, daoAdmin.address);

    // Admin execute dao deploy factory
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

    console.log("\nInstantiate factory");
    let factoryInstRes = await daoClient.executeAdminMsg(deployFactoryMsg);
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
    console.log("\nInstantiate dao_tunnel");
    let daoTunnelInstRes = await daoClient.executeAdminMsg(deployDaoTunnelMsg);
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
    console.log("\nInstantiated remote tunnel: ", remoteTunnelAddr);

    // Admin execute add connection to dao tunnel
    const daoTunnelApproveControllerMsg = daoClient.addApprovedControllerMsg(
        daoTunnelAddr,
        relayerClient.connections.hostConnection,
        `wasm.${remoteTunnelAddr}`
    );
    console.log("\nAdd Connection to dao_tunnel");
    await daoClient.executeAdminMsg(daoTunnelApproveControllerMsg);

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

    const msg = daoClient.executeMsg(daoTunnelAddr, createRemoteFactoryMsg);
    console.log("\nCreate Remote Factory");
    await daoClient.executeAdminMsg(msg);

    // Relay packets and acknowledge
    await relayerClient.relayAll();
    await delay(10000);

    const { remote_factory: remoteFactoryAddr } = await remoteTunnelClient.chainConfig();
    console.log("\nRemote Factory Addr: ", remoteFactoryAddr);

    // Set dao-tunnel address in DAO
    // Update DAO with dao_tunnel addr
    const daoSetItemMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            execute: {
                contract_addr: daoClient.daoAddr,
                funds: [],
                msg: toCosmosMsg({ set_item: { key: "dao-tunnel", addr: daoTunnelAddr } }),
            },
        },
    };

    console.log("\nSet dao_tunnel address in DAO items");
    let res = await daoClient.executeAdminMsg(daoSetItemMsg);

    // Update marketing address on Govec
    res = await govecClient.updateMarketing({
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

    // Add staking address to Govec
    res = await govecClient.updateConfigAddr({
        newAddr: { staking: daoClient.stakingAddr },
    });
    console.log("\n\nUpdated Staking Address on Govec\n", JSON.stringify(res));

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
