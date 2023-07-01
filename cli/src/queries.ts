import { OptionValues } from "commander";
import { Chains } from "./config/chains";

export function handleOptions(opts: OptionValues) {
  if (opts.network) {
    console.log("Supported chains: ", Chains);
  }
}
