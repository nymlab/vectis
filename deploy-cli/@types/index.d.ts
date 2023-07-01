import { VectisHubChainContractsAddrs } from "../interfaces/contracts";
import * as chains from "../config/chains";
declare global {
    var contracts: VectisHubChainContractsAddrs;

    namespace NodeJS {
        interface ProcessEnv {
            HOST_CHAIN: keyof typeof chains;
            REMOTE_CHAIN: keyof typeof chains;
            JUNO_ADMIN_MNEMONIC: string;
            JUNO_ADMIN_ADDRESS: string;
            WASM_ADMIN_MNEMONIC: string;
            WASM_ADMIN_ADDRESS: string;
        }
    }
}

export {};
