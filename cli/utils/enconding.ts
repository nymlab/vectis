import { toBase64, toUtf8 } from "@cosmjs/encoding";

export const toCosmosMsg = <T>(msg: T): string => {
    return toBase64(toUtf8(JSON.stringify(msg)));
};

/// Big endian
export function longToByteArray(long: number): Uint8Array {
    // we want to represent the input as a 8-bytes array
    var byteArray = new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0]);

    for (var index = byteArray.length - 1; index >= 0; index--) {
        var byte = long & 0xff;
        byteArray[index] = byte;
        long = (long - byte) / 256;
    }

    return byteArray;
}
