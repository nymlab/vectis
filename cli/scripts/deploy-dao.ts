import assert from "assert";
import {
    FactoryClient,
    GovecClient,
    DaoClient,
    CWClient,
    Cw3FlexClient,
    Cw4GroupClient,
    PluginRegClient,
    RelayerClient,
} from "../clients";
import { mintableGovecPerWallet } from "../clients/dao";
import { marketingDescription, marketingProject } from "../clients/govec";
import { govecGenesisBalances } from "../config/govec_init_balances";
import { Account } from "../config/accounts";
import { toCosmosMsg } from "../utils/enconding";
import { writeToFile } from "../utils/fs";
import { hostChain, hostChainName, daoDeployReportPath, daoUploadReportPath, hostAccounts } from "../utils/constants";

import type { DaoTunnelT, ProxyT } from "../interfaces";
import type { VectisDaoChainContractsAddrs } from "../interfaces/contracts";
import {
    technicalCommittee1Weight,
    technicalCommittee2Weight,
    preProposalCommitte1Weight,
    preProposalCommitte2Weight,
} from "../clients/dao";
import { DaoActors } from "../utils/constants";

(async function deploy() {
    console.log("Deploy DAO");
    const { factory, proxy, daoTunnel, cw3Fixed, govec, pluginReg, cw4Group, cw3Flex } = await import(
        daoUploadReportPath
    );

    const adminHostClient = await CWClient.connectHostWithAccount("admin");

    // Admin will be removed at the end of the deploy
    const daoAdmin = hostAccounts["admin"] as Account;

    // Check channels config exists with the existing IBC transfer channel
    // used in daoTunnel instantiate msg
    const relayerClient = new RelayerClient();
    const connection = await relayerClient.connect();
    const channels = await relayerClient.loadChannels();
    const { transfer: channelTransfer } = channels.transfer
        ? channels
        : await relayerClient.createChannel("transfer", "transfer", "ics20-1");

    console.log("IBC transfer connections: ", connection, "\n channel:", channelTransfer);

    // ===================================================================================
    //
    // Governance committees:  PreProposal + Tech
    //
    // ===================================================================================

    // These are committee members for both the preproposal and technical committee
    const committee1 = hostAccounts["committee1"] as Account;
    const committee2 = hostAccounts["committee2"] as Account;

    // Instantiate Cw4Group - which will be used for Technical, Trader assignment and Proposal Approval

    // proposal committee group
    const preProposalCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        cw4Group.codeId,
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
    console.log("1. Instantiated group for preproposal committees at: ", proposalCommitteeMembers);

    // tech committee group
    const techCommitteeMemberRes = await Cw4GroupClient.instantiate(
        adminHostClient,
        cw4Group.codeId,
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
    console.log("2. Instantiated group for tech committees at: ", techCommitteeMembers);

    // Instantiate PreProposal MultiSig
    const proposalCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        cw3Flex.codeId,
        proposalCommitteeMembers,
        "Pre Proposal MultiSig"
    );
    const proposalCommittee = proposalCommitteeRes.contractAddress;

    // Instantiate TechCommittee MultiSig
    const techCommitteeRes = await Cw3FlexClient.instantiate(
        adminHostClient,
        cw3Flex.codeId,
        techCommitteeMembers,
        "Tech Committee MultiSig"
    );
    const techCommittee = techCommitteeRes.contractAddress;
    console.log(
        "3. Instantiated Tech committee Multisig at: ",
        techCommittee,
        "Pre Proposal Approval Multisig at: ",
        proposalCommittee
    );

    // Instantiate Govec
    // TODO update token holders
    const govecClient = await GovecClient.instantiate(adminHostClient, govec.codeId, {
        initial_balances: govecGenesisBalances,
        marketing: GovecClient.createVectisMarketingInfo(adminHostClient.sender),
        mintAmount: mintableGovecPerWallet,
    });
    const govecAddr = govecClient.contractAddress;
    console.log("4. Instantiated Govec at: ", govecAddr);

    // ===================================================================================
    //
    // Instantiate DAO and DaoChain contracts
    //
    // ===================================================================================

    // Instantiate DAO with Admin to help with deploy and dao-action tests
    const daoClient = await DaoClient.instantiate(adminHostClient, govecAddr, daoAdmin.address, proposalCommittee);
    console.log("5. DAO at: ", daoClient.daoAddr);

    // Admin execute dao deploy factory
    const factoryInstMsg = FactoryClient.createFactoryInstMsg(hostChainName, proxy.codeId, cw3Fixed.codeId);

    const deployFactoryMsg = {
        wasm: {
            instantiate: {
                admin: daoClient.daoAddr,
                code_id: factory.codeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };

    let factoryInstRes = await daoClient.executeAdminMsg(deployFactoryMsg);
    const factoryAddr = CWClient.getContractAddrFromResult(factoryInstRes, "_contract_address");

    // Admin deploy dao tunnel - connection and channels in config/relayer.ts
    const daoTunnelInstMsg: DaoTunnelT.InstantiateMsg = {
        denom: hostChain.feeToken,
        init_ibc_transfer_mods: { endpoints: [[connection.hostConnection, channelTransfer?.src.channelId as string]] },
    };

    const deployDaoTunnelMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            instantiate: {
                admin: daoClient.daoAddr,
                code_id: daoTunnel.codeId,
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
                code_id: pluginReg.codeId,
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
    console.log("\n7. Set dao_tunnel and committee address in DAO items");

    // Update marketing address on Govec
    let res = await govecClient.updateMarketing({
        marketing: daoClient.daoAddr,
    });
    console.log("\n8. Updated Marketing Address on Govec\n");

    // Update DAO address on Govec
    res = await govecClient.updateDaoAddr({
        newAddr: daoClient.daoAddr,
    });
    console.log("\n9. Updated Dao Address on Govec\n");

    await preProposalCommitteeMemberRes.updateAdmin({ admin: daoClient.daoAddr }, "auto");
    await techCommitteeMemberRes.updateAdmin({ admin: daoClient.daoAddr }, "auto");
    console.log("\n10. Updated preProposal and tech committee Group (cw4)  Contract Admin Role to DAO\n");

    res = await adminHostClient.updateAdmin(
        adminHostClient.sender,
        proposalCommitteeMembers,
        daoClient.daoAddr,
        "auto"
    );
    await adminHostClient.updateAdmin(adminHostClient.sender, govecAddr, daoClient.daoAddr, "auto");
    await adminHostClient.updateAdmin(adminHostClient.sender, techCommitteeMembers, daoClient.daoAddr, "auto");
    await adminHostClient.updateAdmin(adminHostClient.sender, proposalCommittee, daoClient.daoAddr, "auto");
    await adminHostClient.updateAdmin(adminHostClient.sender, techCommittee, daoClient.daoAddr, "auto");
    console.log("\n11. Updated govec, proposal + tech committee (cw4 groups + cw3 flex) Contract Admin to DAO");

    const contracts = {
        // These are the same as DaoActors
        Govec: govecAddr,
        DaoTunnel: daoTunnelAddr,
        ProposalCommittee: proposalCommittee,
        PreProposalModule: daoClient.preProposalAddr,
        PluginCommittee: techCommittee,
        PluginRegistry: pluginRegAddr,
        Factory: factoryAddr,
        Staking: daoClient.stakingAddr,

        // Additional contracts deployed
        Dao: daoClient.daoAddr,
        ProposalModule: daoClient.proposalAddr,
        PreproposalGroup: proposalCommitteeMembers,
        PluginCommitteeGroup: techCommitteeMembers,
        Vote: daoClient.voteAddr,
    };
    console.log("\n Contracts on DaoChain: ", contracts);
    writeToFile(daoDeployReportPath, JSON.stringify(contracts, null, 2));

    await verify(contracts);
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

async function verify(addrs: VectisDaoChainContractsAddrs) {
    console.log("Verifying deployment");
    const adminHostClient = await CWClient.connectHostWithAccount("admin");
    const govecClient = new GovecClient(adminHostClient, adminHostClient.sender, addrs.Govec);

    // Deployer account should not have any govec token.
    const { balance } = await govecClient.balance({
        address: adminHostClient.sender,
    });
    assert.strictEqual(balance, "0");
    console.log("Govec balance of deployer is ZERO");

    // DAO Supply should be sum of gensis valuese
    // TODO: this is currently set to 100 just by default giving to a test "user"
    const tokenInfo = await govecClient.tokenInfo();
    assert.strictEqual(tokenInfo.total_supply, "100");
    console.log("Govec balance strict equal to 100");

    // Checks contract admins for migrations
    for (const [key, value] of Object.entries(addrs)) {
        // Ensure DAO is admin so that it can update contracts.
        if (key != "Dao") {
            let contract = await adminHostClient.getContract(value);
            assert.strictEqual(contract.admin, addrs.Dao);
        } else {
            // DAO should have not have admin
            let contract = await adminHostClient.getContract(value);
            assert.strictEqual(contract.admin, undefined);
        }
    }
    console.log("All contracts has DAO as contract admin");

    // DAO should have all DaoActors addresses
    for (const [key, value] of Object.entries(DaoActors)) {
        // TODO: to add treasury
        if (key != "TreasuryCommittee") {
            let addr = await adminHostClient.queryContractSmart(addrs.Dao, {
                get_item: { key: value },
            });
            assert.strictEqual(addrs[key as keyof typeof addrs], addr.item);
        }
    }
    console.log("DaoActors stored correctly on DAO");

    // Govec minters should be only Dao, DaoTunnel and Factory
    const { minters } = await govecClient.minters();
    console.log("Govec minters: ", minters);
    assert.ok(minters?.includes(addrs.Dao));
    assert.ok(minters?.includes(addrs.DaoTunnel));
    assert.ok(minters?.includes(addrs.Factory));
    assert.equal(minters?.length, 3);

    // Govec should have vectis project and description
    const marketingInfo = await govecClient.marketingInfo();
    assert.strictEqual(marketingInfo.project, marketingProject);
    assert.strictEqual(marketingInfo.description, marketingDescription);
}
