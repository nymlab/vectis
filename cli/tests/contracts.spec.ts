import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";
import { getContract } from "@vectis/core/utils/fs";

import {
    daoTunnelCodetPath,
    factoryCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    proxyCodePath,
    remoteFactoryCodePath,
    remoteProxyCodePath,
    remoteTunnelCodePath,
    uploadReportPath,
} from "@vectis/core/utils/constants";

/**
 * This suite tests contracts upload
 */
describe("Contracts Suite: ", () => {
    it("Should upload contracts", async () => {
        const { host, remote } = await import(uploadReportPath);
        const { factoryRes, proxyRes, daoTunnelRes, multisigRes, govecRes } = host;
        const { remoteTunnel, remoteMultisig, remoteProxy, remoteFactory } = remote;

        const factoryCode = getContract(factoryCodePath);
        const proxyCode = getContract(proxyCodePath);
        const multisigCode = getContract(fixMultiSigCodePath);
        const govecCode = getContract(govecCodePath);
        const daoTunnelCode = getContract(daoTunnelCodetPath);
        const remoteTunnelCode = getContract(remoteTunnelCodePath);
        const remoteFactoryCode = getContract(remoteFactoryCodePath);
        const remoteProxyCode = getContract(remoteProxyCodePath);
        const remoteMultisigCode = getContract(fixMultiSigCodePath);

        // Factory
        expect(factoryRes.originalChecksum).toEqual(toHex(sha256(factoryCode)));
        expect(factoryRes.compressedSize).toBeLessThan(factoryCode.length * 0.5);
        expect(factoryRes.codeId).toBeGreaterThanOrEqual(1);

        // Proxy
        expect(proxyRes.originalChecksum).toEqual(toHex(sha256(proxyCode)));
        expect(proxyRes.compressedSize).toBeLessThan(proxyCode.length * 0.5);
        expect(proxyRes.codeId).toBeGreaterThanOrEqual(1);

        // Multisig
        expect(multisigRes.originalChecksum).toEqual(toHex(sha256(multisigCode)));
        expect(multisigRes.compressedSize).toBeLessThan(multisigCode.length * 0.5);
        expect(multisigRes.codeId).toBeGreaterThanOrEqual(1);

        // Govec
        expect(govecRes.originalChecksum).toEqual(toHex(sha256(govecCode)));
        expect(govecRes.compressedSize).toBeLessThan(govecCode.length * 0.5);
        expect(govecRes.codeId).toBeGreaterThanOrEqual(1);

        // Dao Tunnel
        expect(daoTunnelRes.originalChecksum).toEqual(toHex(sha256(daoTunnelCode)));
        expect(daoTunnelRes.compressedSize).toBeLessThan(daoTunnelCode.length * 0.5);
        expect(daoTunnelRes.codeId).toBeGreaterThanOrEqual(1);

        // Remote Tunnel
        expect(remoteTunnel.originalChecksum).toEqual(toHex(sha256(remoteTunnelCode)));
        expect(remoteTunnel.compressedSize).toBeLessThan(remoteTunnelCode.length * 0.5);
        expect(remoteTunnel.codeId).toBeGreaterThanOrEqual(1);

        // Remote Factory
        expect(remoteFactory.originalChecksum).toEqual(toHex(sha256(remoteFactoryCode)));
        expect(remoteFactory.compressedSize).toBeLessThan(remoteFactoryCode.length * 0.5);
        expect(remoteFactory.codeId).toBeGreaterThanOrEqual(1);

        // Remote Proxy
        expect(remoteProxy.originalChecksum).toEqual(toHex(sha256(remoteProxyCode)));
        expect(remoteProxy.compressedSize).toBeLessThan(remoteProxyCode.length * 0.5);
        expect(remoteProxy.codeId).toBeGreaterThanOrEqual(1);

        // Remote Multisig
        expect(remoteMultisig.originalChecksum).toEqual(toHex(sha256(remoteMultisigCode)));
        expect(remoteMultisig.compressedSize).toBeLessThan(remoteMultisigCode.length * 0.5);
        expect(remoteMultisig.codeId).toBeGreaterThanOrEqual(1);
    });

    // TODO
    it("Should have have the correct checksums for onchain contracts", async () => {});
});
