import assert from "assert";
import {
    FactoryClient,
    GovecClient,
    DaoClient,
    RelayerClient,
    CWClient,
    Cw3FlexClient,
    Cw4GroupClient,
    PluginRegClient,
} from "../clients";
import { marketingDescription, marketingProject } from "../clients/govec";
import RemoteTunnelClient from "../clients/remote-tunnel";
import { govecGenesisBalances } from "../config/govec_init_balances";

import { Account } from "../config/accounts";
import { toCosmosMsg } from "../utils/enconding";
import { writeInCacheFolder } from "../utils/fs";
import { hostChain, hostChainName, remoteChainName, uploadReportPath, hostAccounts } from "../utils/constants";

import type { DaoTunnelT, GovecT, ProxyT, PluginRegT } from "../interfaces";
import type { VectisDaoContractsAddrs } from "../interfaces/contracts";
import {
    technicalCommittee1Weight,
    technicalCommittee2Weight,
    preProposalCommitte1Weight,
    preProposalCommitte2Weight,
} from "../clients/dao";
import { DaoActors, deployReportPath } from "../utils/constants";

// See README.md for deployment info
//
(async function deploy() {
    console.log("Starting to deploy");
    const { host, remote } = await import(uploadReportPath);
    const { factoryRes, proxyRes, daoTunnelRes, multisigRes, govecRes, pluginRegRes, Cw4GroupRes, Cw3FlexRes } = host;
    const { remoteTunnel, remoteMultisig, remoteProxy, remoteFactory } = remote;

    const relayerClient = new RelayerClient();
    const connection = await relayerClient.connect();
    const channels = await relayerClient.loadChannels();
    const { transfer: channelTransfer } = channels.transfer
        ? channels
        : await relayerClient.createChannel("transfer", "transfer", "ics20-1");

    console.log("1. Create IBC transfer channel between chains");
    console.log("ibc transfer channel: ", channelTransfer);
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
    console.log("2. Instantiated Govec at: ", govecAddr);

    // Admin will be removed at the end of the deploy
    const daoAdmin = hostAccounts["admin"] as Account;

    // ===================================================================================
    //
    // Governance committees = PreProposal + Tech
    //
    // ===================================================================================

    // These are committee members for both the preproposal and technical committee
    const committee1 = hostAccounts["committee1"] as Account;
    const committee2 = hostAccounts["committee2"] as Account;

    // Instantiate Cw4Group - which will be used for Technical, Trader assignment and Proposal Approval
    const preProposalCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        Cw4GroupRes.codeId,
        daoAdmin.address,
        [
            {
                addr: committee1.address,
                weight: preProposalCommitte1Weight,
            },
            {
                addr: committee2.address,
                weight: preProposalCommitte2Weight,
            },
        ],
        "Vectis PrePropsoal Committee"
    );
    const proposalCommitteeMembers = preProposalCommitteeMemberRes.contractAddress;
    console.log("3. Instantiated group for preproposal committees at: ", proposalCommitteeMembers);

    const techCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        Cw4GroupRes.codeId,
        daoAdmin.address,
        [
            {
                addr: committee1.address,
                weight: technicalCommittee1Weight,
            },
            {
                addr: committee2.address,
                weight: technicalCommittee2Weight,
            },
        ],
        "Vectis Tech Committee"
    );
    const techCommitteeMembers = techCommitteeMemberRes.contractAddress;
    console.log("3. Instantiated group for tech committees at: ", techCommitteeMembers);

    // Instantiate PreProposal MultiSig
    const proposalCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        Cw3FlexRes.codeId,
        proposalCommitteeMembers,
        "Pre Proposal MultiSig"
    );
    const proposalCommittee = proposalCommitteeRes.contractAddress;

    // Instantiate TechCommittee MultiSig
    const techCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        Cw3FlexRes.codeId,
        techCommitteeMembers,
        "Tech Committee MultiSig"
    );
    const techCommittee = techCommitteeRes.contractAddress;
    console.log(
        "4. Instantiated Tech committee Cw3 at: ",
        techCommittee,
        "Pre Proposal Approval Cw3 at: ",
        proposalCommittee
    );

    // ===================================================================================
    //
    // Instantiate DAO and DaoChain contracts
    //
    // ===================================================================================

    // Instantiate DAO with Admin to help with deploy and dao-action tests
    const daoClient = await DaoClient.instantiate(adminHostClient, govecAddr, daoAdmin.address, proposalCommittee);
    console.log("5. DAO at: ", daoClient.daoAddr);

    // Admin execute dao deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(hostChainName, proxyRes.codeId, multisigRes.codeId);

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

    let factoryInstRes = await daoClient.executeAdminMsg(deployFactoryMsg);
    const factoryAddr = CWClient.getContractAddrFromResult(factoryInstRes, "_contract_address");

    // Admin deploy dao tunnel
    const daoTunnelInstMsg: DaoTunnelT.InstantiateMsg = {
        denom: hostChain.feeToken,
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

    let daoTunnelInstRes = await daoClient.executeAdminMsg(deployDaoTunnelMsg);
    const daoTunnelAddr = CWClient.getContractAddrFromResult(daoTunnelInstRes, "_contract_address");

    // Admin deploy plugin registry
    let pluginRegInstMsg = PluginRegClient.createInstMsg(hostChainName);
    const deployPluginRegistry = {
        wasm: {
            instantiate: {
                admin: daoClient.daoAddr,
                code_id: pluginRegRes.codeId,
                funds: [],
                label: "Vectis Plugin Registry",
                msg: toCosmosMsg(pluginRegInstMsg),
            },
        },
    };

    let regInstRes = await daoClient.executeAdminMsg(deployPluginRegistry);
    const pluginRegAddr = CWClient.getContractAddrFromResult(regInstRes, "_contract_address");

    console.log(
        "\n6. Instantiated dao_tunnel at: ",
        daoTunnelAddr,
        "\n Instantiated factory at: ",
        factoryAddr,
        "\n Instantiated Plugin Reg at: ",
        pluginRegAddr
    );

    // =================================================================================
    //
    // Instantiate Remote chain contracts
    //
    // =================================================================================

    // Instantiate Remote Tunnel
    const remoteTunnelClient = await RemoteTunnelClient.instantiate(adminRemoteClient, remoteTunnel.codeId, {
        dao_config: {
            addr: daoClient.daoAddr,
            dao_tunnel_port_id: `wasm.${daoTunnelAddr}`,
            connection_id: relayerClient.connections.remoteConnection,
        },
        init_ibc_transfer_mod: {
            endpoints: [[connection.remoteConnection, channelTransfer?.dest.channelId as string]],
        },
    });

    const remoteTunnelAddr = remoteTunnelClient.contractAddress;
    console.log("\n7. Instantiated remote tunnel at: ", remoteTunnelAddr);

    // Admin execute add connection to dao tunnel
    const daoTunnelApproveControllerMsg = daoClient.addApprovedControllerMsg(
        daoTunnelAddr,
        relayerClient.connections.hostConnection,
        `wasm.${remoteTunnelAddr}`
    );
    await daoClient.executeAdminMsg(daoTunnelApproveControllerMsg);
    console.log("\n8. Add Approved Connection (remote tunnel addr) to dao_tunnel");

    // Instantiate Factory in remote chain

    // Create channel
    const { wasm: channelWasm } = await relayerClient.createChannel(
        `wasm.${daoTunnelAddr}`,
        `wasm.${remoteTunnelAddr}`,
        "vectis-v1"
    );

    console.log("\n9. Relayer create channel between remote and dao tunnel: ", channelWasm);

    // Admin execute dao deploy remote factory
    let hostBlock = await govecClient.client.getBlock();
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
    await daoClient.executeAdminMsg(msg);

    // Relay packets and acknowledge
    await relayerClient.runRelayerWithAck("host", hostBlock.header.height);

    const remoteFactoryAddr = await remoteTunnelClient.getItem({ key: "Factory" });
    console.log("\n10. Instantiate remote chain Factory at ", remoteFactoryAddr);

    // ===================================================================================
    //
    // Set addresses on DaoChain DAO and other contracts not instantiated by DAO
    //
    // ===================================================================================
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.Govec, govecAddr));
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.DaoTunnel, daoTunnelAddr));
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.ProposalCommittee, proposalCommittee));
    await daoClient.executeAdminMsg(
        dao_set_item(daoClient.daoAddr, DaoActors.PreProposalModule, daoClient.preProposalAddr)
    );
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.PluginCommittee, techCommittee));
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.PluginRegistry, pluginRegAddr));
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.Factory, factoryAddr));
    await daoClient.executeAdminMsg(dao_set_item(daoClient.daoAddr, DaoActors.Staking, daoClient.stakingAddr));
    // TODO treasury
    console.log("\n12. Set dao_tunnel and committee address in DAO items");

    // Update marketing address on Govec
    let res = await govecClient.updateMarketing({
        marketing: daoClient.daoAddr,
    });
    console.log("\n13. Updated Marketing Address on Govec\n", JSON.stringify(res));

    // Update DAO address on Govec
    res = await govecClient.updateDaoAddr({
        newAddr: daoClient.daoAddr,
    });
    console.log("\n14. Updated Dao Address on Govec\n", JSON.stringify(res));

    res = await adminHostClient.updateAdmin(adminHostClient.sender, govecAddr, daoClient.daoAddr, "auto");
    console.log("\n15. Updated Govec Contract Admin to DAO\n", JSON.stringify(res));

    let respp = await preProposalCommitteeMemberRes.updateAdmin({ admin: daoClient.daoAddr }, "auto");
    let rest = await techCommitteeMemberRes.updateAdmin({ admin: daoClient.daoAddr }, "auto");
    console.log(
        "\n16. Updated preProposal and tech committee Group (cw4) Admin Role to DAO\n",
        JSON.stringify(respp),
        "\n",
        JSON.stringify(rest)
    );
    res = await adminHostClient.updateAdmin(
        adminHostClient.sender,
        proposalCommitteeMembers,
        daoClient.daoAddr,
        "auto"
    );
    res = await adminHostClient.updateAdmin(adminHostClient.sender, techCommitteeMembers, daoClient.daoAddr, "auto");
    res = await adminHostClient.updateAdmin(adminHostClient.sender, proposalCommittee, daoClient.daoAddr, "auto");
    res = await adminHostClient.updateAdmin(adminHostClient.sender, techCommittee, daoClient.daoAddr, "auto");
    console.log("\n17. Updated proposal + tech committee (cw4 groups + cw3 flex) Contract Admin to DAO");

    const contracts = {
        remoteFactoryAddr: remoteFactoryAddr as string,
        remoteTunnelAddr,
        daoTunnelAddr,
        daoAddr: daoClient.daoAddr,
        govecAddr,
        factoryAddr,
        stakingAddr: daoClient.stakingAddr,
        proposalAddr: daoClient.proposalAddr,
        preproposalAddr: daoClient.preProposalAddr,
        preProposalMultiSigAddr: proposalCommittee,
        preproposalGroupAddr: proposalCommitteeMembers,
        pluginRegistryAddr: pluginRegAddr,
        techCommitteeMultiSigAddr: techCommittee,
        techCommitteeGroupAddr: techCommitteeMembers,
        voteAddr: daoClient.voteAddr,
    };
    console.log("\n Contracts: ", contracts);
    writeInCacheFolder("deployInfo.json", JSON.stringify(contracts, null, 2));

    await verifyDeploy(contracts);
    console.log("\nEND. Verified deployment");
})();

function dao_set_item(contract_addr: string, key: DaoActors, value: string): ProxyT.CosmosMsgForEmpty {
    return {
        wasm: {
            execute: {
                contract_addr,
                funds: [],
                msg: toCosmosMsg({ set_item: { key, value } }),
            },
        },
    };
}

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

    // Ensure DAO is admin so that it can update.
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

    // TODO: DAO should have all DaoActors addresses

    // Govec should have factory addr and dao_tunnel addr
    const { minters } = await govecClient.minters();
    console.log("minters: ", minters);
    assert.ok(minters?.includes(addrs.daoAddr));
    assert.ok(minters?.includes(addrs.daoTunnelAddr));
    assert.ok(minters?.includes(addrs.factoryAddr));
    assert.equal(minters?.length, 3);

    // Govec should have have vectis project and description
    const marketingInfo = await govecClient.marketingInfo();
    assert.strictEqual(marketingInfo.project, marketingProject);
    assert.strictEqual(marketingInfo.description, marketingDescription);
}
