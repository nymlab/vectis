import { OptionValues } from "commander";
import * as chains from "./config/chains";

export function handleOptions(opts: OptionValues) {
    if (opts.network) {
        console.log("Supported chains: ", Object.keys(chains));
    }
}
