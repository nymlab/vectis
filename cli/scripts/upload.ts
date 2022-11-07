import assert from "assert";
import { sha256 } from "@cosmjs/crypto";
import { toHex } from "@cosmjs/encoding";

import CosmWasmClient, { downloadContracts } from "../clients/cosmwasm";
import { areContractsDownloaded, getContract, writeInCacheFolder } from "../utils/fs";
import {
    daoTunnelCodetPath,
    factoryCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    hostChain,
    proxyCodePath,
    remoteChain,
    remoteFactoryCodePath,
    remoteProxyCodePath,
    remoteTunnelCodePath,
} from "../utils/constants";

import type { ContractsResult } from "../interfaces/contracts";

async function uploadCode() {
    if (!areContractsDownloaded()) await downloadContracts();

    const daoClient = await CosmWasmClient.connectWithAccount(hostChain, "admin");
    const uploadHostRes = await daoClient.uploadHostContracts(hostChain);

    const remoteClient = await CosmWasmClient.connectWithAccount(remoteChain, "admin");
    const uploadRemoteRes = await remoteClient.uploadRemoteContracts();

    const results = { host: { ...uploadHostRes }, remote: { ...uploadRemoteRes } } as ContractsResult;

    await verifyUpload(results);

    writeInCacheFolder("uploadInfo.json", JSON.stringify(results, null, 2));
}

async function verifyUpload({ host, remote }: ContractsResult) {
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
    assert.strictEqual(factoryRes.originalChecksum, toHex(sha256(factoryCode)));
    assert.strictEqual(factoryRes.compressedSize < factoryCode.length * 0.5, true);
    assert.strictEqual(factoryRes.codeId >= 1, true);

    // Proxy
    assert.strictEqual(proxyRes.originalChecksum, toHex(sha256(proxyCode)));
    assert.strictEqual(proxyRes.compressedSize < proxyCode.length * 0.5, true);
    assert.strictEqual(proxyRes.codeId >= 1, true);

    // Multisig
    assert.strictEqual(multisigRes.originalChecksum, toHex(sha256(multisigCode)));
    assert.strictEqual(multisigRes.compressedSize < multisigCode.length * 0.5, true);
    assert.strictEqual(multisigRes.codeId >= 1, true);

    // Govec
    assert.strictEqual(govecRes.originalChecksum, toHex(sha256(govecCode)));
    assert.strictEqual(govecRes.compressedSize < govecCode.length * 0.5, true);
    assert.strictEqual(govecRes.codeId >= 1, true);

    // Dao Tunnel
    assert.strictEqual(daoTunnelRes.originalChecksum, toHex(sha256(daoTunnelCode)));
    assert.strictEqual(daoTunnelRes.compressedSize < daoTunnelCode.length * 0.5, true);
    assert.strictEqual(daoTunnelRes.codeId >= 1, true);

    // Remote Tunnel
    assert.strictEqual(remoteTunnel.originalChecksum, toHex(sha256(remoteTunnelCode)));
    assert.strictEqual(remoteTunnel.compressedSize < remoteTunnelCode.length * 0.5, true);
    assert.strictEqual(remoteTunnel.codeId >= 1, true);

    // Remote Factory
    assert.strictEqual(remoteFactory.originalChecksum, toHex(sha256(remoteFactoryCode)));
    assert.strictEqual(remoteFactory.compressedSize < remoteFactoryCode.length * 0.5, true);
    assert.strictEqual(remoteFactory.codeId >= 1, true);

    // Remote Proxy
    assert.strictEqual(remoteProxy.originalChecksum, toHex(sha256(remoteProxyCode)));
    assert.strictEqual(remoteProxy.compressedSize < remoteProxyCode.length * 0.5, true);
    assert.strictEqual(remoteProxy.codeId >= 1, true);

    // Remote Multisig
    assert.strictEqual(remoteMultisig.originalChecksum, toHex(sha256(remoteMultisigCode)));
    assert.strictEqual(remoteMultisig.compressedSize < remoteMultisigCode.length * 0.5, true);
    assert.strictEqual(remoteMultisig.codeId >= 1, true);
}

uploadCode();
