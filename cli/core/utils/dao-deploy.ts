import { createSigningClient } from "../services/cosmwasm";
import { createVectisMarketingInfo, instantiateGovec } from "../services/govec";
import { addrPrefix, adminAddr, adminMnemonic, uploadReportPath } from "./constants";
import { GovecClient, GovecT, ProxyT } from "@vectis/types";
import { createTokenInfo } from "@vectis/core/services/staking";
import {
    createDaoInstMsg,
    createGovModInstInfo,
    createPropInstMsg,
    createVoteInstMsg,
    createVoteModInstInfo,
} from "@vectis/core/services/dao";
import { defaultExecuteFee, defaultInstantiateFee } from "@vectis/core/utils/fee";
// import { delay } from "@vectis/core/utils/promises";
import {
    threshold,
    depositInfo,
    maxVotingPeriod,
    minVotingPeriod,
    allowRevote,
    unstakeDuration,
    walletFee,
} from "@vectis/core/utils/dao-params";
import { toCosmosMsg } from "@vectis/core/utils/enconding";
import { createFactoryInstMsg } from "../services/factory";
import { VectisDaoContractsAddrs } from "../interfaces/dao";
import {
    ExecuteMsg as CwPropSingleExecuteMsg,
    QueryMsg as ProposalQueryMsg,
} from "@dao-dao/types/contracts/cw-proposal-single";

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
export async function deploy(): Promise<VectisDaoContractsAddrs> {
    const { factoryRes, proxyRes, multisigRes, stakingRes, voteRes, govecRes, daoRes, proposalSingleRes } =
        await import(uploadReportPath);

    const adminClient = await createSigningClient(adminMnemonic, addrPrefix);
    const initial_balances: GovecT.Cw20Coin[] = [
        {
            address: adminAddr,
            amount: "1",
        },
    ];

    // Instantiate Govec
    const { govecAddr } = await instantiateGovec({
        client: adminClient,
        initial_balances,
        govecCodeId: govecRes.codeId as number,
        admin: adminAddr,
        marketing: createVectisMarketingInfo(adminAddr),
    });

    const govecClient = new GovecClient(adminClient, adminAddr, govecAddr);
    console.log("Instantiated Govec at: ", govecAddr);

    // Instantiate DAO
    const tokenInfo = createTokenInfo(govecAddr, stakingRes.id, unstakeDuration);
    const voteInstMsg = createVoteInstMsg(tokenInfo);
    // cw-proposal-single instantiation msg
    const propInstMsg = createPropInstMsg(depositInfo, maxVotingPeriod, minVotingPeriod, threshold, allowRevote);
    // dao-core instantiation msg
    // TODO: the module types `ModuleInstantiateInfo` do not work with the @daodao/types,
    // therefore not using interfaces. There is versioning issues
    // https://github.com/DA0-DA0/dao-contracts/pull/347#pullrequestreview-1011556931
    const govModInstInfo = createGovModInstInfo(proposalSingleRes.id, propInstMsg);
    const voteModInstInfo = createVoteModInstInfo(voteRes.id, voteInstMsg);
    const daoInstMsg = createDaoInstMsg(govModInstInfo, voteModInstInfo);

    console.log("dao code id: ", daoRes.id);
    const { contractAddress: daoAddr } = await adminClient.instantiate(
        adminAddr,
        daoRes.id,
        daoInstMsg,
        "VectisDAO",
        defaultInstantiateFee
    );

    const voteAddr = await adminClient.queryContractSmart(daoAddr, { voting_module: {} });
    const [proposalAddr] = await adminClient.queryContractSmart(daoAddr, { proposal_modules: {} });
    const stakingAddr = await adminClient.queryContractSmart(voteAddr, { staking_contract: {} });
    console.log("Instantiated DAO at: ", daoAddr);
    console.log("Instantiated Vote at: ", voteAddr);
    console.log("Instantiated proposal at: ", proposalAddr);
    console.log("Instantiated staking at: ", stakingAddr);

    // Admin stake initial balance to prepare for proposal
    await govecClient.updateStakingAddr({ newAddr: stakingAddr });
    const sendMsg = { stake: {} };
    await govecClient.send({ amount: "1", contract: stakingAddr, msg: toCosmosMsg(sendMsg) });

    // Admin propose and execute dao deploy factory
    const factoryInstMsg = createFactoryInstMsg(proxyRes.codeId, multisigRes.codeId, addrPrefix, walletFee, govecAddr);
    const deployFactoryMsg: ProxyT.CosmosMsgForEmpty = {
        wasm: {
            instantiate: {
                admin: daoAddr,
                code_id: factoryRes.codeId,
                funds: [],
                label: "Vectis Factory",
                msg: toCosmosMsg(factoryInstMsg),
            },
        },
    };

    const proposal: CwPropSingleExecuteMsg = {
        propose: {
            description: "Deploy Vectis Factory",
            latest: null,
            msgs: [deployFactoryMsg],
            title: "Deploy Vectis Factory",
        },
    };

    // Propose
    let res = await adminClient.execute(adminAddr!, proposalAddr, proposal, defaultExecuteFee);
    const propQuery: ProposalQueryMsg = { list_proposals: {} };
    const props = await adminClient.queryContractSmart(proposalAddr, propQuery);
    const proposalId = props.proposals[0].id;
    console.log("\n\nProposed to deploy Factory Contract\n", JSON.stringify(res));

    // Vote and Execute to deploy Factory
    const vote: CwPropSingleExecuteMsg = {
        vote: {
            proposal_id: proposalId,
            vote: "yes",
        },
    };
    res = await adminClient.execute(adminAddr, proposalAddr, vote, defaultExecuteFee);
    console.log("\n\nVote to deploy Factory Contract\n", JSON.stringify(res));

    const execute: CwPropSingleExecuteMsg = {
        execute: {
            proposal_id: proposalId,
        },
    };
    res = await adminClient.execute(adminAddr!, proposalAddr, execute, defaultExecuteFee);
    const events = res.logs[0].events; // Wasm event is always the last
    const attributes = events[events.length - 1].attributes;
    const factoryEvent = attributes.find((ele) => ele.key == "Vectis Factory instantiated");
    console.log("\n\nExecuted Proposal to deploy Factory\n", JSON.stringify(res));

    // Update marketing address on Govec
    res = await govecClient.updateMarketing({ marketing: daoAddr });
    console.log("\n\nUpdated Marketing Address on Govec\n", JSON.stringify(res));

    // Update staking address on Govec
    res = await govecClient.updateStakingAddr({ newAddr: stakingAddr });
    console.log("\n\nUpdated Staking Address on Govec\n", JSON.stringify(res));

    // Update minter address on Govec to Factory
    res = await govecClient.updateMintData({ newMint: { minters: [factoryEvent!.value] } });
    console.log("\n\nUpdated Minter Address on Govec\n", JSON.stringify(res));

    // Update DAO address on Govec
    res = await govecClient.updateDaoAddr({ newAddr: daoAddr });
    console.log("\n\nUpdated Dao Address on Govec\n", JSON.stringify(res));

    res = await adminClient.updateAdmin(adminAddr!, govecAddr, daoAddr, defaultExecuteFee);
    console.log("\n\nUpdated Govec Contract Admin to DAO\n", JSON.stringify(res));

    res = await adminClient.execute(adminAddr!, stakingAddr, { unstake: { amount: "1" } }, defaultExecuteFee);
    console.log("\n\nAdmin unstakes \n", JSON.stringify(res));

    //// Below is only needed if theres is an unstake period
    // delay(5000);
    // res = await adminClient.execute(adminAddr!, stakingAddr, { claim: {} }, defaultExecuteFee);
    // console.log("\n\nAdmin claim \n", JSON.stringify(res));

    res = await adminClient.execute(adminAddr!, govecAddr, { burn: {} }, defaultExecuteFee);
    console.log("\n\nAdmin burns the one govec\n", JSON.stringify(res));

    return {
        daoAddr: daoAddr,
        govecAddr: govecAddr,
        factoryAddr: factoryEvent!.value,
        stakingAddr: stakingAddr,
        proposalAddr: proposalAddr,
        voteAddr: voteAddr,
    };
}
