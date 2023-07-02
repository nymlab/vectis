import figlet from "figlet";
import { Command } from "commander";
import { uploadAction } from "./upload";
import { deploy } from "./deploy";
import { handleOptions } from "./queries";

console.log(figlet.textSync("Vectis"));

const program = new Command();
program.option("-n | --networks", "Supported networks").action(handleOptions);

program
    .command("upload")
    .argument("<network>", "Network to add contract(s) to")
    .argument("[contract...]", "Array of contracts to upload, default: all")
    .action(uploadAction);

program.command("deploy").argument("<network>", "Network to deploy to").action(deploy);

program.parse();
