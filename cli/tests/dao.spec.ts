import { FactoryClient, GovecClient, CWClient, DaoClient, ProxyClient } from "../clients";
import { hostAccounts, hostChain } from "../utils/constants";
import { getDefaultWalletCreationFee, walletInitialFunds } from "../utils/fees";
import { coin } from "@cosmjs/stargate";
import { Coin } from "../interfaces/Factory.types";
import { DaoTunnelClient } from "../interfaces";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { deployReportPath } from "../utils/constants";
import { delay } from "../utils/promises";

/**
 * This suite tests deployment scripts for deploying Vectis as a sovereign DAO
 */

describe("DAO Suite:", () => {
    let addrs: VectisDaoContractsAddrs;
    let adminClient: CWClient;
    let userClient: CWClient;
    let proxyClient: ProxyClient;
    let govecClient: GovecClient;
    let daoClient: DaoClient;
    let daoTunnelClient: DaoTunnelClient;
    let factoryClient: FactoryClient;

    beforeAll(async () => {
        addrs = await import(deployReportPath);
        adminClient = await CWClient.connectHostWithAccount("admin");
        userClient = await CWClient.connectHostWithAccount("user");
        govecClient = new GovecClient(adminClient, adminClient.sender, addrs.govecAddr);
        factoryClient = new FactoryClient(userClient, userClient.sender, addrs.factoryAddr);
        daoTunnelClient = new DaoTunnelClient(adminClient, adminClient.sender, addrs.daoTunnelAddr);
        daoClient = new DaoClient(adminClient, {
            daoAddr: addrs.daoAddr,
            proposalAddr: addrs.proposalAddr,
            stakingAddr: addrs.stakingAddr,
            voteAddr: addrs.voteAddr,
        });

        const initialFunds = walletInitialFunds(hostChain);
        const walletCreationFee = await factoryClient.fee();
        const totalFee: Number = Number(walletCreationFee.amount) + Number(initialFunds.amount);

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
        await proxyClient.mintGovec(addrs.factoryAddr);
        await proxyClient.stakeGovec(addrs.govecAddr, addrs.stakingAddr, "1");
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

    it.skip("DAO should be able to add approved controllers in dao_tunnel", async () => {
        const { tunnels } = await daoTunnelClient.controllers({});

        const daoTunnelApproveControllerMsg = daoClient.createApprovedControllerMsg(
            addrs.daoTunnelAddr,
            "1",
            `wasm.addr`
        );
        await daoClient.createProposal("Allow connection in DAO Tunnel", "Allow connection in DAO Tunnel", [
            daoTunnelApproveControllerMsg,
        ]);
        const { proposals } = await daoClient.queryProposals();
        const approveControllerProposalId = proposals.length;
        await delay(10000);

        await daoClient.voteProposal(approveControllerProposalId, "yes");
        await delay(10000);
        await daoClient.executeProposal(approveControllerProposalId);
        await delay(10000);

        let res = await daoTunnelClient.controllers({});

        expect(tunnels.length).toBe(res.tunnels.length + 1);
    });

    afterAll(() => {
        adminClient?.disconnect();
    });
});
