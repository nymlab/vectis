import { coin } from "@cosmjs/amino";
import { CWClient, GovecClient, RelayerClient } from "../clients";
import RemoteProxyClient from "../clients/remote-proxy";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { Coin } from "../interfaces/Factory.types";
import { RemoteFactoryClient } from "../interfaces/RemoteFactory.client";
import { QueryMsg as prePropQueryMsg, CosmosMsgForEmpty } from "../interfaces/DaoPreProposeApprovalSingel.types";
import { deployReportPath, hostChain, remoteChain } from "../utils/constants";
import { generateRandomAddress } from "./mocks/addresses";
import { createSingleProxyWallet } from "./mocks/proxyWallet";

describe("Proxy Remote Suite: ", () => {
    let addrs: VectisDaoContractsAddrs;
    let userClient: CWClient;
    let hostUserClient: CWClient;
    let factoryClient: RemoteFactoryClient;
    let proxyClient: RemoteProxyClient;
    let govecClient: GovecClient;
    const relayerClient = new RelayerClient();
    beforeAll(async () => {
        addrs = await import(deployReportPath);
        await relayerClient.connect();
        await relayerClient.loadChannels();
        userClient = await CWClient.connectRemoteWithAccount("user");
        hostUserClient = await CWClient.connectHostWithAccount("user");
        factoryClient = new RemoteFactoryClient(userClient, userClient.sender, addrs.remoteFactoryAddr);
        govecClient = new GovecClient(hostUserClient, hostUserClient.sender, addrs.govecAddr);
        const walletAddr = await createSingleProxyWallet(factoryClient, "remote");
        proxyClient = new RemoteProxyClient(userClient, userClient.sender, walletAddr!);
    });

    it("should be able to do ibc transfer", async () => {
        const amount = "1000000";
        const address = await generateRandomAddress("juno");
        const previousBalance = await hostUserClient.getBalance(address, relayerClient.denoms.dest);

        expect(previousBalance.amount).toBe("0");

        await proxyClient.sendIbcTokens(addrs.remoteTunnelAddr, address, relayerClient.connections.remoteConnection, [
            coin(amount, remoteChain.feeToken) as Coin,
        ]);

        await relayerClient.runRelayerWithoutAck("remote", null);

        const currentBalance = await hostUserClient.getBalance(address, relayerClient.denoms.dest);

        expect(currentBalance.amount).toBe(amount);
    });

    it("should be able to mint govec", async () => {
        const { claim_fee } = await factoryClient.fees();
        let unclaimedBefore = await factoryClient.unclaimedGovecWallets({});
        await proxyClient.mintGovec(addrs.remoteFactoryAddr, claim_fee);
        await relayerClient.runRelayerWithAck("remote", null);
        const { accounts } = await govecClient.allAccounts({ limit: 50 });
        const { balance } = await govecClient.balance({ address: proxyClient.contractAddress });
        expect(accounts.includes(proxyClient.contractAddress)).toBeTruthy();
        expect(balance).toBe("2");
    });

    it("should be able to transfer govec", async () => {
        const { balance: previousBalance } = await govecClient.balance({ address: addrs.daoAddr });
        const { balance: contractPreviousBalance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });

        await proxyClient.transferGovec(addrs.remoteTunnelAddr, addrs.daoAddr, "1");
        await relayerClient.runRelayerWithoutAck("remote", null);

        const { balance: currentBalance } = await govecClient.balance({ address: addrs.daoAddr });
        const { balance: contractCurrentBalance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(-+previousBalance + +currentBalance).toBe(1);
        expect(+contractPreviousBalance + -+contractCurrentBalance).toBe(1);
    });

    it("should be able to stake", async () => {
        let stakeRes = await proxyClient.stakeGovec(addrs.remoteTunnelAddr, addrs.stakingAddr, "1");
        await relayerClient.runRelayerWithoutAck("remote", null);
        const { value } = await hostUserClient.queryContractSmart(addrs.stakingAddr, {
            staked_value: { address: proxyClient.contractAddress },
        });
        expect(value).toBe("1");
    });

    it("should be able to unstake", async () => {
        await proxyClient.unstakeGovec(addrs.remoteTunnelAddr, "1");
        await relayerClient.runRelayerWithoutAck("remote", null);

        const { value } = await hostUserClient.queryContractSmart(addrs.stakingAddr, {
            staked_value: { address: proxyClient.contractAddress },
        });
        expect(value).toBe("0");
    });
});

describe("Proxy Remote (Proposal flow) suite: ", () => {
    let addrs: VectisDaoContractsAddrs;
    let userClient: CWClient;
    let hostUserClient: CWClient;
    let factoryClient: RemoteFactoryClient;
    let proxyClient: RemoteProxyClient;
    let govecClient: GovecClient;
    const relayerClient = new RelayerClient();
    beforeAll(async () => {
        addrs = await import(deployReportPath);
        await relayerClient.connect();
        await relayerClient.loadChannels();
        userClient = await CWClient.connectRemoteWithAccount("user");
        hostUserClient = await CWClient.connectHostWithAccount("user");
        factoryClient = new RemoteFactoryClient(userClient, userClient.sender, addrs.remoteFactoryAddr);
        govecClient = new GovecClient(hostUserClient, hostUserClient.sender, addrs.govecAddr);
        const walletAddr = await createSingleProxyWallet(factoryClient, "remote");
        proxyClient = new RemoteProxyClient(userClient, userClient.sender, walletAddr!);
    });

    it("should be able to mint govec only once", async () => {
        const { claim_fee } = await factoryClient.fees();
        await proxyClient.mintGovec(addrs.remoteFactoryAddr, claim_fee);
        await relayerClient.runRelayerWithAck("remote", null);

        const { accounts } = await govecClient.allAccounts({ limit: 50 });
        const { balance } = await govecClient.balance({ address: proxyClient.contractAddress });
        expect(accounts.includes(proxyClient.contractAddress)).toBeTruthy();
        expect(balance).toBe("2");

        // must fail if we can mint again
        await expect(proxyClient.mintGovec(addrs.remoteFactoryAddr, claim_fee)).rejects.toBeInstanceOf(Error);
    });

    it("should be able to do pre proposal", async () => {
        await hostUserClient.sendTokens(
            hostUserClient.sender,
            addrs.daoAddr,
            [coin("1000000", hostChain.feeToken) as Coin],
            "auto"
        );

        // order is desending
        let queryMsg: prePropQueryMsg = { query_extension: { msg: { pending_proposals: {} } } };
        const previousPreProposals = await hostUserClient.queryContractSmart(addrs.preproposalAddr, queryMsg);
        const msg: CosmosMsgForEmpty = {
            bank: {
                send: {
                    from_address: addrs.daoAddr,
                    to_address: addrs.daoTunnelAddr,
                    amount: [coin("100000", hostChain.feeToken) as Coin],
                },
            },
        };

        let res = await proxyClient.createPreProposal(
            addrs.remoteTunnelAddr,
            addrs.preproposalAddr,
            "title_proposal",
            "description_proposal",
            [msg]
        );

        await relayerClient.runRelayerWithoutAck("remote", null);

        let currentPreProposals = await hostUserClient.queryContractSmart(addrs.preproposalAddr, queryMsg);

        const { approval_id, proposer, msg: prepmsg, deposit } = currentPreProposals[0];

        expect(previousPreProposals.length + 1).toBe(currentPreProposals.length);
        expect(proposer).toBe(proxyClient.contractAddress);
    });
});
