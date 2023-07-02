import type { UploadResult } from "@cosmjs/cosmwasm-stargate";

export enum VectisContracts {
    "factory",
    "proxy",
    "pluginReg",
    "cronkitty",
    "cw3Fixed",
    "cw3Flex",
    "cw4Group",
}

export type Contract = keyof typeof VectisContracts;
