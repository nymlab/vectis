import { OptionValues } from "commander";
import * as chains from "./config/chains";

export function handleOptions(opts: OptionValues) {
    if (opts.networks) {
        console.log(Object.keys(chains));
    }
}
