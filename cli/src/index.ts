import figlet from "figlet";
import { Command } from "commander";
import { uploadAction } from "./upload";
import { test, test_query } from "./test";
import { deploy } from "./deploy-vectis";
import { handleAccounts } from "./accounts";
import { handleNetworkQuery } from "./queries";
import { generateTypes } from "./types";
import { factory } from "./factory";

console.log(figlet.textSync("Vectis"));

const program = new Command();

program
    .command("upload")
    .argument("<network>", "Network to add contract(s) to")
    .option("--contracts <contracts...>", "Array of contracts to upload, default: all")
    .option("--vectis", "Upload vectis contracts")
    .action(uploadAction);

program.command("test").argument("<network>", "Network to test create and send tx").action(test);

program
    .command("factory")
    .argument("<network>", "Network")
    .option("--action [action]", "Action to perform")
    .option("--args [args...]")
    .description(
        "e.g. vectisCLI factory juno_testnet --action updateSupportedChains --args theta-testnet-001 ibc connection-697"
    )
    .action(factory);

program.command("generateAccounts").option("--network [networks...]", "Network to generate").action(handleAccounts);

program.command("deployVectis").argument("<network>", "Network deploy Vectis to for testing").action(deploy);

program.command("networks").description("Query the supported networks").action(handleNetworkQuery);

program.command("types").description("generate types for vectis contracts").action(generateTypes);

//program
//    .command("list-scw")
//    .description("List all Vectis accounts by the controller address")
//    .argument("<network>", "Network to look for contracts")
//    .argument("controller", "Name of controller of Vectis Account");
//
//program
//    .command("query-scw")
//    .description("Get `WalletInfo` of a given Vectis Account")
//    .argument("<network>", "Network to look for contracts")
//    .argument("account", "Address of the Vectis Account to query");
//
//program
//    .command("install-plugin")
//    .description("Install a plugin to a wallet")
//    .argument("<network>", "Network to instantiate contract")
//    .argument("controller", "Name of controller of Vectis Account")
//    .argument("wallet", "Wallet address")
//    .argument("<instantiateMsg>", "Message for the plugin")
//    .option("--registry <registered-id>")
//    .option("--unregistry <code-id>");
//
program.parse();
