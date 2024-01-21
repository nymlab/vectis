import { Chains } from "./config/chains";
import { CWClient, ProxyClient, FactoryClient } from "./clients";
import { Logger } from "tslog";
import { getDeployPath } from "./utils/fs";
import { delay } from "./utils/promises";
import { toCosmosMsg } from "./utils/enconding";
import { OptionValues } from "commander";
import { VectisContractsAddrs } from "./interfaces";
import { toUtf8, toBase64, fromBase64, fromUtf8 } from "@cosmjs/encoding";
import { coins, StdFee } from "@cosmjs/stargate";

const pubkey = new Uint8Array([
    4, 254, 213, 81, 121, 242, 209, 178, 171, 160, 209, 220, 243, 199, 156, 57, 7, 187, 116, 219, 198, 101, 89, 52, 55,
    116, 76, 44, 30, 67, 0, 143, 189, 75, 244, 25, 219, 51, 204, 90, 94, 118, 253, 230, 111, 25, 66, 150, 185, 16, 177,
    143, 185, 58, 174, 105, 199, 187, 209, 50, 112, 128, 88, 201, 199,
]);

const vectisRelayTxStr =
    '{"messages":[{"bank":{"send":{"amount":[{"denom":"ujunox","amount":"10"}],"to_address":"juno1qc6cq2lsd0vccceups73auddtu2p6pymwd6ush"}}}],"nonce":0}';

const asn1Signature = new Uint8Array([
    48, 70, 2, 33, 0, 239, 139, 59, 204, 38, 219, 76, 13, 81, 227, 244, 206, 106, 106, 82, 233, 224, 66, 106, 231, 148,
    119, 165, 59, 129, 5, 37, 86, 23, 26, 127, 230, 2, 33, 0, 245, 7, 2, 167, 86, 219, 132, 126, 6, 203, 61, 98, 248, 2,
    14, 132, 111, 193, 212, 143, 212, 201, 248, 197, 239, 30, 189, 97, 112, 61, 42, 195,
]);

// auth_data: [u8; 37]
const authData = new Uint8Array([
    160, 69, 228, 49, 29, 7, 49, 76, 44, 221, 70, 108, 153, 137, 118, 53, 175, 165, 26, 158, 250, 12, 31, 58, 208, 251,
    254, 192, 151, 172, 43, 29, 1, 0, 0, 0, 0,
]);

// new Uint8Array(response.clientDataJSON))
const clientData = new Uint8Array([
    123, 34, 116, 121, 112, 101, 34, 58, 34, 119, 101, 98, 97, 117, 116, 104, 110, 46, 103, 101, 116, 34, 44, 34, 99,
    104, 97, 108, 108, 101, 110, 103, 101, 34, 58, 34, 90, 71, 87, 84, 81, 66, 48, 116, 116, 81, 72, 112, 114, 83, 68,
    121, 117, 121, 97, 118, 81, 118, 70, 76, 107, 116, 79, 77, 109, 97, 85, 97, 121, 85, 103, 112, 52, 112, 97, 77, 72,
    122, 107, 34, 44, 34, 111, 114, 105, 103, 105, 110, 34, 58, 34, 104, 116, 116, 112, 115, 58, 47, 47, 112, 97, 115,
    115, 107, 101, 121, 45, 112, 111, 99, 46, 118, 101, 114, 99, 101, 108, 46, 97, 112, 112, 34, 44, 34, 99, 114, 111,
    115, 115, 79, 114, 105, 103, 105, 110, 34, 58, 102, 97, 108, 115, 101, 125,
]);

export async function test_query(network: Chains) {
    const logger = new Logger();
    const client = await CWClient.connectHostWithAccount("admin", network);
    const uploadedContracts: VectisContractsAddrs = await import(getDeployPath(network));

    let query = {
        authenticate: {
            signed_data: Array.from(toUtf8(vectisRelayTxStr)),
            controller_data: Array.from(pubkey),
            metadata: Array.from([Array.from(authData), Array.from(clientData)]),
            signature: Array.from(asn1Signature),
        },
    };

    let queryRes = await client.client.queryContractSmart(uploadedContracts.Webauthn, query);

    logger.info("Query Result: ", JSON.stringify(queryRes));
}

export async function test(network: Chains, opts: OptionValues) {
    const logger = new Logger();
    const client = await CWClient.connectHostWithAccount("walletCreator", network);

    const uploadedContracts: VectisContractsAddrs = await import(getDeployPath(network));
    const factoryClient = new FactoryClient(client, client.sender, uploadedContracts.Factory);
    const dataKey = toCosmosMsg("Some-key");
    const dataValue = toCosmosMsg("Some-value");
    let res = await factoryClient.createWalletWebAuthn(
        pubkey,
        [[dataKey, dataValue]],
        "test-hash-2", // the hash of the label / display name
        [], // this is plugins to instantiate, we will keep as none for now
        [{ denom: "ujunox", amount: "100000" }], // initial proxy token
        [client.sender]
    );
    console.log("create wallet res: \n", JSON.stringify(res));

    let proxyAddr = CWClient.getEventAttrValue(res, "wasm-vectis.proxy.v1", "_contract_address");
    logger.info("proxy Addr: ", proxyAddr);

    let proxyClient = new ProxyClient(client, client.sender, proxyAddr);
    let data = await proxyClient.data({ key: dataKey });
    logger.info("\n\nproxy data: ", fromUtf8(fromBase64(data!)));

    let info = await proxyClient.info();
    logger.info("\n\nproxy info: ", info);

    // Check current balance on the proxy
    let currentBalance = await proxyClient.client.getBalance(proxyAddr, "ujunox");
    logger.info("\n\ncurrentBalance: ", currentBalance);

    await proxyClient.relayTxFromSelf(
        vectisRelayTxStr,
        toBase64(authData),
        toBase64(clientData),
        toBase64(asn1Signature)
    );

    // wait for block
    await delay(8000);

    let afterBalance = await proxyClient.client.getBalance(proxyAddr, "ujunox");
    logger.info("\n\nafterBalance: ", afterBalance);

    let infoAfter = await proxyClient.info();
    logger.info("\n\nproxy info after tx: ", infoAfter);

    ////============== show fee grant usage ==================

    let recipient = await CWClient.connectHostWithAccount("user", network);

    let fee: StdFee = { amount: coins("2000", "ujunox"), gas: "80000", granter: proxyAddr };

    let testFeeGrant = await client.client.sendTokens(
        client.sender,
        recipient.sender,
        [{ denom: "ujunox", amount: "10" }],
        fee
    );
    logger.info("testFeeGrant: ", JSON.stringify(testFeeGrant));

    // Check current balance on the proxy
    let afterTestFeeGrantBalance = await proxyClient.client.getBalance(proxyAddr, "ujunox");
    logger.info("\n\nafterTestFeeGrantBalance: ", afterTestFeeGrantBalance);
}
