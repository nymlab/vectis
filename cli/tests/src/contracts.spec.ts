import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import { getContract } from "@vectis/core/utils/fs";

import {
    factoryCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    proxyCodePath,
    uploadReportPath,
} from "@vectis/core/utils/constants";

/**
 * This suite tests contracts upload
 */
describe("Contracts Suite: ", () => {
    it("Should upload contracts", async () => {
        const { factoryRes, proxyRes, multisigRes, govecRes } = await import(uploadReportPath);

        const factoryCode = getContract(factoryCodePath);
        const proxyCode = getContract(proxyCodePath);
        const multisigCode = getContract(fixMultiSigCodePath);
        const govecCode = getContract(govecCodePath);

        expect(factoryRes.originalChecksum).toEqual(toHex(sha256(factoryCode)));
        expect(factoryRes.compressedSize).toBeLessThan(factoryCode.length * 0.5);
        expect(factoryRes.codeId).toBeGreaterThanOrEqual(1);
        expect(proxyRes.originalChecksum).toEqual(toHex(sha256(proxyCode)));
        expect(proxyRes.compressedSize).toBeLessThan(proxyCode.length * 0.5);
        expect(proxyRes.codeId).toBeGreaterThanOrEqual(1);
        expect(multisigRes.originalChecksum).toEqual(toHex(sha256(multisigCode)));
        expect(multisigRes.compressedSize).toBeLessThan(multisigCode.length * 0.5);
        expect(multisigRes.codeId).toBeGreaterThanOrEqual(1);
        expect(govecRes.originalChecksum).toEqual(toHex(sha256(govecCode)));
        expect(govecRes.compressedSize).toBeLessThan(govecCode.length * 0.5);
        expect(govecRes.codeId).toBeGreaterThanOrEqual(1);
    });

    // TODO
    it("Should have have the correct checksums for onchain contracts", async () => {});
});
