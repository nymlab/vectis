import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import { getContract } from "@vectis/core/utils/fs";

import {
    factoryCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    proxyCodePath,
    stakingCodePath,
} from "@vectis/core/utils/constants";

/**
 * This suite tests contracts upload and deploy
 */
describe("Contracts Suite: ", () => {
    it("Should upload contracts", async () => {
        const { factoryRes, proxyRes, multisigRes, govecRes, stakingRes } = await import(
            "../../contractAddresses.json" as string
        );

        const factoryCode = getContract(factoryCodePath);
        const proxyCode = getContract(proxyCodePath);
        const multisigCode = getContract(fixMultiSigCodePath);
        const govecCode = getContract(govecCodePath);
        const stakingCode = getContract(stakingCodePath);

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
        expect(stakingRes.originalChecksum).toEqual(toHex(sha256(stakingCode)));
        expect(stakingRes.compressedSize).toBeLessThan(stakingCode.length * 0.5);
        expect(stakingRes.codeId).toBeGreaterThanOrEqual(1);
    });
});
