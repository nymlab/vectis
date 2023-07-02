import { OptionValues } from "commander";
import * as chains from "./config/chains";

export function handleOptions(opts: OptionValues) {
    console.log(opts);
    if (opts.networks) {
        console.log("Supported chains: ", Object.keys(chains));
    }
}
