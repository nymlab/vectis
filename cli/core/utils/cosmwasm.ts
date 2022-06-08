import { toBase64, toUtf8 } from "@cosmjs/encoding";

export const toCosmosMsg = <T>(msg: T): string => {
    return toBase64(toUtf8(JSON.stringify(msg)));
};
