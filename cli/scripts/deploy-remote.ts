import { RelayerClient, CWClient } from "../clients";
import RemoteTunnelClient from "../clients/remote-tunnel";
import { writeToFile } from "../utils/fs";
import {
    daoDeployReportPath,
    remoteDeployReportPath,
    remoteUploadReportPath,
    remoteChainName,
} from "../utils/constants";

(async function deploy() {
    console.log("Deploy Remote to: ", remoteChainName);

    const { remoteTunnel } = await import(remoteUploadReportPath);
    const { Dao, DaoTunnel } = await import(daoDeployReportPath);

    const relayerClient = new RelayerClient();
    const connection = await relayerClient.connect();
    const channels = await relayerClient.loadChannels();
    const { transfer: channelTransfer } = channels.transfer
        ? channels
        : await relayerClient.createChannel("transfer", "transfer", "ics20-1");
    console.log("IBC transfer connections: ", connection, "\n channel:", channelTransfer);
    const adminRemoteClient = await CWClient.connectRemoteWithAccount("admin");

    // Instantiate Remote Tunnel
    const remoteTunnelClient = await RemoteTunnelClient.instantiate(adminRemoteClient, remoteTunnel.codeId, {
        dao_config: {
            addr: Dao,
            dao_tunnel_port_id: `wasm.${DaoTunnel}`,
            connection_id: relayerClient.connections.remoteConnection,
        },
        init_ibc_transfer_mod: {
            endpoints: [[connection.remoteConnection, channelTransfer?.dest.channelId as string]],
        },
    });

    const remoteTunnelAddr = remoteTunnelClient.contractAddress;
    console.log("\n1. Instantiated remote tunnel at: ", remoteTunnelAddr);

    const contracts = {
        remoteFactoryAddr: "",
        remoteTunnelAddr,
    };
    console.log("\n Contracts: ", contracts);
    writeToFile(remoteDeployReportPath, JSON.stringify(contracts, null, 2));

    // Then we will need to go through governance to connect with the DaoTunnel
})();
