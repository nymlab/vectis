import { FactoryClient, GovecClient, CWClient, DaoClient, ProxyClient, RelayClient } from "../clients";
import { DaoActors, hostChain, remoteChain, uploadReportPath } from "../utils/constants";
import { coin } from "@cosmjs/stargate";
import { DaoTunnelClient } from "../interfaces";
import { VectisDaoContractsAddrs } from "../interfaces/contracts";
import { deployReportPath } from "../utils/constants";
import { delay } from "../utils/promises";
import { ExecuteMsg as DaoTunnelExecuteMsg } from "../interfaces/DaoTunnel.types";
import { CosmosMsgForEmpty } from "../interfaces/Proxy.types";
import { toCosmosMsg } from "../utils/enconding";

/**
 * This suite tests the deployed sets of contracts Vectis has to represent a sovereign DAO
 */

describe("DAO Suite for Config Tests:", () => {
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

    const relayerClient = new RelayClient();
    beforeAll(async () => {
        addrs = await import(deployReportPath);
        const { host } = await import(uploadReportPath);

        await relayerClient.connect();
        await relayerClient.loadChannels();

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

        await daoClient.executeAdminMsg(daoTunnelApproveControllerMsg);

        let res = await daoTunnelClient.controllers({});

        expect(tunnels.length + 1).toBe(res.tunnels.length);
    });

    it("DAO should be able to update dao config in remote_tunnel", async () => {
        const newAddr = "addr";
        const { addr: addressPrevious } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            dao_config: {},
        });

        await changeRemoteTunnelDaoAddr(newAddr);

        const { addr: addressAfter } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            dao_config: {},
        });

        expect(addressAfter).toBe(newAddr);

        // Revert changes

        await changeRemoteTunnelDaoAddr(addressPrevious);

        const { addr } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            dao_config: {},
        });

        expect(addr).toBe(addressPrevious);
    });

    it("DAO should be able to update ibc transfer channel on remote-tunnel", async () => {
        const { endpoints: channelPrevious } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            ibc_transfer_channels: {},
        });

        const doesIncludeDefaultChannel = channelPrevious.some((c: string[]) =>
            c.includes(relayerClient.channels.transfer?.dest.channelId as string)
        );

        expect(doesIncludeDefaultChannel).toBeTruthy();

        const randomChannel = "channel-" + Math.random().toString(36).slice(2, 9);

        await changeIbcTransferChannel(randomChannel);

        const { endpoints: channelAfter } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            ibc_transfer_channels: {},
        });

        console.log("endpoint: ", JSON.stringify(channelAfter));

        const doesIncludeRandomChannel = channelAfter.some((c: string[]) => c.includes(randomChannel));

        expect(doesIncludeRandomChannel).toBeTruthy();

        // Revert changes

        await changeIbcTransferChannel(relayerClient.channels.transfer?.dest.channelId as string);

        const { endpoints: channel } = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            ibc_transfer_channels: {},
        });

        const doesIncludeDefaultChannelAfter = channel.some((c: string[]) =>
            c.includes(relayerClient.channels.transfer?.dest.channelId as string)
        );

        expect(doesIncludeDefaultChannelAfter).toBeTruthy();
    });

    it("DAO should be able to dispatch an update item on remote_tunnel", async () => {
        const previousAddr = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            get_item: { key: DaoActors.Factory },
        });
        const new_factory = "new-factory";

        await changeRemoteTunnelItem(DaoActors.Factory, new_factory);

        const currentAddr = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            get_item: { key: DaoActors.Factory },
        });
        expect(currentAddr).not.toBe(previousAddr);

        // Revert changes

        await changeRemoteTunnelItem(DaoActors.Factory, previousAddr);

        const revertedConfig = await userRemoteClient.queryContractSmart(addrs.remoteTunnelAddr, {
            get_item: { key: DaoActors.Factory },
        });
        expect(revertedConfig).toStrictEqual(previousAddr);
    });

    it("DAO should be able to change configuration on the remote factory", async () => {
        const newCodeId = 1;
        const amount = (0.1e6).toString();
        const denom = "denom";

        const { codeId: oldCodeId, wallet_fee: oldWalletFee } = await queryRemoteFactoryConfig();

        // we update the wallet fee
        const msgs = createMsgsForUpdateRemoteFactoryConfig(newCodeId, amount, denom);
        await changeRemoteFactoryConfig(msgs);

        const { codeId: currentCodeId, wallet_fee: currentWalletFee } = await queryRemoteFactoryConfig();

        expect(currentCodeId).toBe(newCodeId);
        expect(currentWalletFee.amount).toBe(amount);
        expect(currentWalletFee.denom).toBe(denom);

        // Revert changes
        const revertMsgs = createMsgsForUpdateRemoteFactoryConfig(oldCodeId, oldWalletFee.amount, oldWalletFee.denom);
        await changeRemoteFactoryConfig(revertMsgs);

        const { codeId, wallet_fee } = await queryRemoteFactoryConfig();

        expect(codeId).toBe(oldCodeId);
        expect(wallet_fee.amount).toBe(oldWalletFee.amount);
        expect(wallet_fee.denom).toBe(oldWalletFee.denom);
    });

    it("DAO should be able to remove approved controllers in dao_tunnel", async () => {
        const { tunnels } = await daoTunnelClient.controllers({});

        const msg = daoClient.removeApprovedControllerMsg(addrs.daoTunnelAddr, randomConnection, `wasm.addr`);

        await daoClient.executeAdminMsg(msg);
        await delay(8000);
        await relayerClient.relayAll();

        let res = await daoTunnelClient.controllers({});
        expect(tunnels.length - 1).toBe(res.tunnels.length);
    });

    it("DAO should be able to update proxy code id in Dao-chain factory", async () => {
        const codeId = 0;
        const oldCodeId = await factoryClient.codeId({ ty: "proxy" });

        expect(oldCodeId).not.toBe(codeId);

        const msg = daoClient.executeMsg(addrs.factoryAddr, {
            update_code_id: {
                new_code_id: codeId,
                type: "proxy",
            },
        });
        await daoClient.executeAdminMsg(msg);
        const newCodeId = await factoryClient.codeId({ ty: "proxy" });
        expect(newCodeId).toBe(codeId);

        // revert changes
        const rmsg = daoClient.executeMsg(addrs.factoryAddr, {
            update_code_id: {
                new_code_id: oldCodeId,
                type: "proxy",
            },
        });
        await daoClient.executeAdminMsg(rmsg);
        const currentCodeId = await factoryClient.codeId({ ty: "proxy" });
        expect(currentCodeId).toBe(oldCodeId);
    });

    it("DAO should be able to update wallet fee in Dao-chain factory", async () => {
        const fee = coin(1010, hostChain.feeToken);
        const { wallet_fee: oldFee } = await factoryClient.fees();

        expect(oldFee).not.toStrictEqual(fee);

        const msg = daoClient.executeMsg(addrs.factoryAddr, {
            update_config_fee: {
                new_fee: fee,
                type: "wallet",
            },
        });

        await daoClient.executeAdminMsg(msg);
        const { wallet_fee: newFee } = await factoryClient.fees();
        expect(newFee).toStrictEqual(fee);

        // revert changes
        const rmsg = daoClient.executeMsg(addrs.factoryAddr, {
            update_config_fee: {
                new_fee: oldFee,
                type: "wallet",
            },
        });

        let res = await daoClient.executeAdminMsg(rmsg);
        const { wallet_fee: currentFee } = await factoryClient.fees();
        expect(currentFee).toStrictEqual(oldFee);
    });

    it("DAO should be able to update govec addr in Dao-chain factory", async () => {
        const account = await CWClient.generateRandomAccount(hostChain.addressPrefix);
        const [{ address }] = await account.getAccounts();
        const govecAddr = address;
        const oldGovec = await daoClient.item(DaoActors.Govec);

        expect(oldGovec.item).not.toBe(govecAddr);

        let msg = daoClient.executeMsg(addrs.daoAddr, {
            set_item: {
                key: DaoActors.Govec,
                value: govecAddr,
            },
        });
        await daoClient.executeAdminMsg(msg);
        const newGovec = await daoClient.item(DaoActors.Govec);
        expect(newGovec.item).toBe(govecAddr);

        // revert changes
        msg = daoClient.executeMsg(addrs.daoAddr, {
            set_item: {
                key: DaoActors.Govec,
                value: oldGovec.item,
            },
        });
        await daoClient.executeAdminMsg(msg);

        const currentGovec = await daoClient.item(DaoActors.Govec);
        expect(currentGovec.item).toBe(oldGovec.item);
    });

    const changeRemoteFactoryConfig = async (msgs: CosmosMsgForEmpty[]) => {
        const msg: DaoTunnelExecuteMsg = {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 5,
                msg: {
                    dispatch_actions: {
                        msgs,
                    },
                },
            },
        };

        const updateDaoConfigMsg = daoClient.executeMsg(addrs.daoTunnelAddr, msg);
        await daoClient.executeAdminMsg(updateDaoConfigMsg);
        await delay(10000);
        await relayerClient.relayAll();
    };

    const changeIbcTransferChannel = async (channel: string) => {
        const msg: DaoTunnelExecuteMsg = {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 5,
                msg: {
                    update_ibc_transfer_reciever_channel: {
                        channel: channel,
                        connection_id: relayerClient.connections.remoteConnection as string,
                    },
                },
            },
        };
        const changeIbcTransferMsg = daoClient.executeMsg(addrs.daoTunnelAddr, msg);
        await daoClient.executeAdminMsg(changeIbcTransferMsg);
        await relayerClient.relayAll();
        await delay(8000);
    };

    const queryRemoteFactoryConfig = async () => {
        const codeId = await userRemoteClient.queryContractSmart(addrs.remoteFactoryAddr, {
            code_id: {
                ty: "proxy",
            },
        });

        const { wallet_fee } = await userRemoteClient.queryContractSmart(addrs.remoteFactoryAddr, {
            fees: {},
        });

        return { wallet_fee, codeId };
    };

    const createMsgsForUpdateRemoteFactoryConfig = (newCodeId: number, amount: string, denom: string) => {
        return [
            {
                update_code_id: {
                    new_code_id: newCodeId,
                    type: "proxy",
                },
            },
            {
                update_config_fee: {
                    new_fee: {
                        amount,
                        denom,
                    },
                    type: "wallet",
                },
            },
        ].map((msg) => ({
            wasm: {
                execute: {
                    contract_addr: addrs.remoteFactoryAddr,
                    msg: toCosmosMsg(msg),
                    funds: [],
                },
            },
        }));
    };

    const changeRemoteTunnelItem = async (key: string, value: string | null) => {
        const msg: DaoTunnelExecuteMsg = {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 5,
                msg: {
                    set_item: {
                        key,
                        value,
                    },
                },
            },
        };

        const updateDaoConfigMsg = daoClient.executeMsg(addrs.daoTunnelAddr, msg);
        await daoClient.executeAdminMsg(updateDaoConfigMsg);
        await delay(8000);
        await relayerClient.relayAll();
    };

    const changeRemoteTunnelDaoAddr = async (newAddr: string) => {
        const msg: DaoTunnelExecuteMsg = {
            dispatch_action_on_remote_tunnel: {
                channel_id: relayerClient.channels.wasm?.src.channelId as string,
                job_id: 5,
                msg: {
                    update_dao_config: {
                        new_config: {
                            addr: newAddr,
                            connection_id: relayerClient.connections.remoteConnection,
                            dao_tunnel_channel: relayerClient.channels.wasm?.dest.channelId,
                            dao_tunnel_port_id: `wasm.${addrs.daoTunnelAddr}`,
                        },
                    },
                },
            },
        };

        const updateDaoConfigMsg = daoClient.executeMsg(addrs.daoTunnelAddr, msg);
        await daoClient.executeAdminMsg(updateDaoConfigMsg);
        await delay(8000);
        await relayerClient.relayAll();
    };

    afterAll(async () => {
        adminClient?.disconnect();
    });
});
