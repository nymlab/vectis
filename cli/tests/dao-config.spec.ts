import { FactoryClient, GovecClient, CWClient, DaoClient, ProxyClient, RelayClient } from "../clients";
import { hostAccounts, hostChain, remoteChain } from "../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import {
    BankExtension,
    coin,
    QueryClient,
    setupBankExtension,
    setupStakingExtension,
    SigningStargateClient,
    StakingExtension,
} from "@cosmjs/stargate";
import { Coin } from "../interfaces/Factory.types";
import { DaoTunnelClient, DaoTunnelT } from "../interfaces";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { deployReportPath } from "../utils/constants";
import { delay } from "../utils/promises";
import { Tendermint34Client } from "@cosmjs/tendermint-rpc";
import long from "long";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite:", () => {
    const randomConnection = "connection-" + Math.random().toString(36).slice(2, 9);
    let addrs: VectisDaoContractsAddrs;
    let adminClient: CWClient;
    let userClient: CWClient;
    let userRemoteClient: CWClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let daoTunnelClient: DaoTunnelClient;
    let factoryClient: FactoryClient;
    let relayerClient: RelayClient;
    let queryClient: QueryClient & StakingExtension & BankExtension;
    let remoteQueryClient: QueryClient & StakingExtension & BankExtension;
    SigningStargateClient;
    beforeAll(async () => {
        queryClient = await QueryClient.withExtensions(
            await Tendermint34Client.connect(hostChain.rpcUrl),
            setupStakingExtension,
            setupBankExtension
        );
        remoteQueryClient = await QueryClient.withExtensions(
            await Tendermint34Client.connect(remoteChain.rpcUrl),
            setupStakingExtension,
            setupBankExtension
        );
        addrs = await import(deployReportPath);
        adminClient = await CWClient.connectHostWithAccount("admin");
        userClient = await CWClient.connectHostWithAccount("user");
        userRemoteClient = await CWClient.connectRemoteWithAccount("user");
        govecClient = new GovecClient(adminClient, adminClient.sender, addrs.govecAddr);
        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.factoryAddr);
        daoTunnelClient = new DaoTunnelClient(adminClient, adminClient.sender, addrs.daoTunnelAddr);
        daoClient = new DaoClient(adminClient, {
            daoAddr: addrs.daoAddr,
            proposalAddr: addrs.proposalAddr,
            stakingAddr: addrs.stakingAddr,
            voteAddr: addrs.voteAddr,
        });
        relayerClient = new RelayClient();
        await relayerClient.connect();
        await relayerClient.loadChannels();

        const initialFunds = walletInitialFunds(hostChain);
        const { wallet_fee } = await factoryClient.fees();
        const totalFee: Number = Number(wallet_fee.amount) + Number(initialFunds.amount);

        const { amount: remainingBalance } = await userClient.getBalance(userClient.sender, remoteChain.feeToken);
        console.log("remaining balance ", remainingBalance);
        console.log("initial", totalFee);
        await factoryClient.createWallet(
            {
                createWalletMsg: {
                    user_addr: userClient.sender,
                    label: "user-wallet",
                    guardians: {
                        addresses: [hostAccounts.guardian_1.address, hostAccounts.guardian_2.address],
                    },
                    relayers: [hostAccounts.relayer_1.address],
                    proxy_initial_funds: [initialFunds],
                },
            },
            getDefaultWalletCreationFee(hostChain),
            undefined,
            [coin(totalFee.toString(), hostChain.feeToken) as Coin]
        );

        const { wallets } = await factoryClient.unclaimedGovecWallets({});
        const walletAddr = wallets[0][0];
        proxyClient = new ProxyClient(userClient, userClient.sender, walletAddr);
        const { claim_fee } = await factoryClient.fees();
        await proxyClient.mintGovec(addrs.factoryAddr, claim_fee);
        await proxyClient.stakeGovec(addrs.govecAddr, addrs.stakingAddr, "2");
        await delay(10000);
    });

    it("DAO should be the only one authorized for communicate with dao_tunnel", async () => {
        const msgAuthorize = {
            add_approved_controller: {
                connection_id: 1,
                port_id: `wasm.address`,
            },
        };

        await adminClient
            .execute(adminClient.sender, addrs.daoTunnelAddr, msgAuthorize, "auto")
            .catch((err: Error) => expect(err).toBeInstanceOf(Error));
    });

    it("DAO should be able to add approved controllers in dao_tunnel", async () => {
        const { tunnels } = await daoTunnelClient.controllers({});

        const daoTunnelApproveControllerMsg = daoClient.addApprovedControllerMsg(
            addrs.daoTunnelAddr,
            randomConnection,
            `wasm.addr`
        );

        await proxyClient.createProposal(
            daoClient.proposalAddr,
            "Allow connection in DAO Tunnel",
            "Allow connection in DAO Tunnel",
            [daoTunnelApproveControllerMsg]
        );
        await delay(10000);

        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;

        await proxyClient.voteProposal(daoClient.proposalAddr, approveControllerProposalId, "yes");
        await delay(10000);

        await proxyClient.executeProposal(daoClient.proposalAddr, approveControllerProposalId);
        await delay(10000);

        let res = await daoTunnelClient.controllers({});

        expect(tunnels.length + 1).toBe(res.tunnels.length);
    });

    it("DAO should be able to remove approved controllers in dao_tunnel", async () => {
        const { tunnels } = await daoTunnelClient.controllers({});

        const msg = daoClient.removeApprovedControllerMsg(addrs.daoTunnelAddr, randomConnection, `wasm.addr`);

        await proxyClient.createProposal(
            daoClient.proposalAddr,
            "Remove connection in DAO Tunnel",
            "Remove connection in DAO Tunnel",
            [msg]
        );
        await delay(10000);

        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;

        await proxyClient.voteProposal(daoClient.proposalAddr, approveControllerProposalId, "yes");
        await delay(10000);

        await proxyClient.executeProposal(daoClient.proposalAddr, approveControllerProposalId);
        await delay(10000);

        let res = await daoTunnelClient.controllers({});

        expect(tunnels.length - 1).toBe(res.tunnels.length);
    });

    it("DAO should be able to update proxy code id in factory", async () => {
        const codeId = 0;
        const oldCodeId = await factoryClient.codeId({ ty: "proxy" });

        expect(oldCodeId).not.toBe(codeId);

        const msg = daoClient.executeMsg(addrs.factoryAddr, {
            update_code_id: {
                new_code_id: codeId,
                ty: "proxy",
            },
        });

        await proxyClient.createProposal(daoClient.proposalAddr, "Update proxy code id", "Update proxy code id", [msg]);
        await delay(10000);

        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;

        await proxyClient.voteProposal(daoClient.proposalAddr, approveControllerProposalId, "yes");
        await delay(10000);

        await proxyClient.executeProposal(daoClient.proposalAddr, approveControllerProposalId);
        await delay(10000);

        const newCodeId = await factoryClient.codeId({ ty: "proxy" });

        expect(newCodeId).toBe(codeId);
    });

    it("DAO should be able to dispatch an update of dao config to remote_tunnel through dao_tunnel", async () => {
        const msg = daoClient.executeMsg(addrs.daoTunnelAddr, {
            dispatch_action_on_remote_tunnel: {
                channel_id: "",
                job_id: 5,
                msg: {
                    update_dao_config: {
                        new_config: {
                            addr: daoClient.daoAddr,
                            connection_id: randomConnection,
                            dao_tunnel_port_id: "",
                        },
                    },
                },
            },
        });
        // TODO  check
    });

    it("DAO should be able to update wallet fee in Dao-chain factory", async () => {
        const fee = coin(1000, hostChain.feeToken);
        const { wallet_fee: oldFee } = await factoryClient.fees();

        expect(oldFee).not.toBe(fee);

        const msg = daoClient.executeMsg(addrs.factoryAddr, {
            update_config_fee: {
                new_fee: { wallet: fee },
            },
        });

        await proxyClient.createProposal(daoClient.proposalAddr, "Update wallet fee", "Update wallet fee", [msg]);
        await delay(8000);

        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;

        await proxyClient.voteProposal(daoClient.proposalAddr, approveControllerProposalId, "yes");
        await delay(10000);

        await proxyClient.executeProposal(daoClient.proposalAddr, approveControllerProposalId);

        const { wallet_fee: newFee } = await factoryClient.fees();

        expect(newFee).toBe(fee);
    });

    it("DAO should be able to govec addr in factory", async () => {
        const account = await CWClient.generateRandomAccount(hostChain.addressPrefix);
        const [{ address }] = await account.getAccounts();
        const govecAddr = address;
        const oldGovec = await factoryClient.govecAddr();

        expect(oldGovec).not.toBe(govecAddr);

        const msg = daoClient.executeMsg(addrs.factoryAddr, {
            update_govec_addr: {
                addr: govecAddr,
            },
        });

        await proxyClient.createProposal(daoClient.proposalAddr, "Update govec addr", "Update govec addr", [msg]);
        await delay(8000);

        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;

        await proxyClient.voteProposal(daoClient.proposalAddr, approveControllerProposalId, "yes");
        await delay(8000);

        await proxyClient.executeProposal(daoClient.proposalAddr, approveControllerProposalId);

        const newGovec = await factoryClient.govecAddr();

        expect(newGovec).toBe(govecAddr);
    });

    afterAll(async () => {
        await proxyClient.unstakeGovec(addrs.stakingAddr, "2");
        await proxyClient.burnGovec(addrs.govecAddr);
        const { balance } = await govecClient.balance({
            address: proxyClient.contractAddress,
        });
        expect(balance).toBe("0");
        adminClient?.disconnect();
    });
});
