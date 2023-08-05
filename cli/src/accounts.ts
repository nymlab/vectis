import { SupportChains, Chains } from "./config/chains";
import { Logger } from "tslog";
import { OptionValues } from "commander";
import { generateAddrFromMnemonics } from "./utils/accounts";

export async function handleAccounts(opts: OptionValues) {
    const logger = new Logger();
    logger.info("opts?", opts);
    let networks: string[];
    if (!opts.network) {
        logger.info("Generating Accounts for All config networks: ");
        networks = Object.keys(SupportChains).filter((n) => isNaN(Number(n)));
    } else {
        networks = opts.network;
    }
    console.log(networks);
    networks.map((n) => {
        generateAddrFromMnemonics(n as Chains);
    });
}
