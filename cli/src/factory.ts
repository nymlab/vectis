import { CWClient, FactoryClient, Cw3FlexClient } from "./clients";
import { Logger } from "tslog";
import { getDeployPath, getAccountsPath } from "./utils/fs";
import { Chains } from "./config/chains";
import { Account } from "./config/accounts";
import { OptionValues } from "commander";
import { VectisContractsAddrs, FactoryT } from "./interfaces";
import { FactoryAction } from "./config/actions/factory";

export async function factory(network: Chains, opts: OptionValues) {
    const logger = new Logger();
    const action = opts.action as FactoryAction;
    const args = opts.args;

    switch (action) {
        case "updateSupportedChains": {
            logger.info("Updating Supported Chains");
            logger.info("Updating chain id: ", args[0]);
            logger.info("Updating connection type:", args[1]);
            logger.info("Updating connection:", args[2]);

            let connection: FactoryT.ChainConnection;
            switch (args[1]) {
                case "ibc": {
                    connection = { i_b_c: args[2] };
                    break;
                }
                case "other": {
                    connection = { other: args[2] };
                    break;
                }
                default: {
                    logger.fatal("supported arg chain connection type");
                    return;
                }
            }

            const client = await CWClient.connectHostWithAccount("admin", network);
            const uploadedContracts: VectisContractsAddrs = await import(getDeployPath(network));
            const factoryClient = new FactoryClient(client, client.sender, uploadedContracts.Factory);
            const hostAccounts: Record<string, Account> = await import(getAccountsPath(network));
            const committee1Client = await CWClient.connectHostWithAccount("committee1", network);
            const committee1 = hostAccounts["committee1"] as Account;

            const vectisComClient = new Cw3FlexClient(
                committee1Client,
                committee1.address,
                uploadedContracts.VectisCommittee
            );
            await vectisComClient.update_supported_chain(args[0], connection, uploadedContracts.Factory);

            let supported = await factoryClient.supportedChains({});
            logger.info("Updated Supported Chains to :", supported);

            break;
        }
        default: {
            logger.info("Unsupported action: ", action);
            break;
        }
    }
}
