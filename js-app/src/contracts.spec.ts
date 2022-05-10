import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { uploadContracts } from "./util/contracts";
import { createSigningClient, getContract } from "./util/utils";

import {
    addrPrefix,
    adminMnemonic,
    factoryCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    proxyCodePath,
    stakingCodePath,
} from "./util/env";

/**
 * This suite tests contracts upload and deploy
 */
// describe("Contracts Suite: ", () => {
//     let adminClient: SigningCosmWasmClient;
//
//     beforeAll(async () => {
//         adminClient = await createSigningClient(adminMnemonic!, addrPrefix!);
//     });
//
//     it("Should upload contracts", async () => {
//         const { factoryRes, proxyRes, multisigRes, govecRes, stakingRes  } = await uploadContracts(adminClient);
//         const factoryCode = getContract(factoryCodePath!);
//         const proxyCode = getContract(proxyCodePath!);
//         const multisigCode = getContract(fixMultiSigCodePath!);
//         const govecCode = getContract(govecCodePath!);
//         const stakingCode = getContract(stakingCodePath!);
//
//         expect(factoryRes.originalChecksum).toEqual(toHex(sha256(factoryCode)));
//         expect(factoryRes.compressedSize).toBeLessThan(factoryCode.length * 0.5);
//         expect(factoryRes.codeId).toBeGreaterThanOrEqual(1);
//         expect(proxyRes.originalChecksum).toEqual(toHex(sha256(proxyCode)));
//         expect(proxyRes.compressedSize).toBeLessThan(proxyCode.length * 0.5);
//         expect(proxyRes.codeId).toBeGreaterThanOrEqual(1);
//         expect(multisigRes.originalChecksum).toEqual(toHex(sha256(multisigCode)));
//         expect(multisigRes.compressedSize).toBeLessThan(multisigCode.length * 0.5);
//         expect(multisigRes.codeId).toBeGreaterThanOrEqual(1);
//         expect(govecRes.originalChecksum).toEqual(toHex(sha256(govecCode)));
//         expect(govecRes.compressedSize).toBeLessThan(govecCode.length * 0.5);
//         expect(govecRes.codeId).toBeGreaterThanOrEqual(1);
//         expect(stakingRes.originalChecksum).toEqual(toHex(sha256(stakingCode)));
//         expect(stakingRes.compressedSize).toBeLessThan(stakingCode.length * 0.5);
//         expect(stakingRes.codeId).toBeGreaterThanOrEqual(1);
//     });
//
//     afterAll(() => {
//         adminClient?.disconnect();
//     });
// });
