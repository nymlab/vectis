import { createSigningClient, downloadContracts, uploadContracts } from "./services/cosmwasm";
import { instantiateFactoryContract } from "./services/factory";
import { instantiateGovec } from "./services/govec";
import { addrPrefix, adminAddr, adminMnemonic } from "./utils/constants";
import { areContractsDownloaded, writeInCacheFolder } from "./utils/fs";
import { FactoryClient } from "@vectis/types/contracts/FactoryContract";

async function main() {
    if (!areContractsDownloaded()) await downloadContracts();

    const adminClient = await createSigningClient(adminMnemonic, addrPrefix);
    const uploadRes = await uploadContracts(adminClient);

    const { govecRes, factoryRes, proxyRes, multisigRes } = uploadRes;

    const { factoryAddr } = await instantiateFactoryContract(
        adminClient,
        factoryRes.codeId,
        proxyRes.codeId,
        multisigRes.codeId
    );
    const { govecAddr } = await instantiateGovec(adminClient, govecRes.codeId, factoryAddr);

    const factoryClient = new FactoryClient(adminClient, adminAddr, factoryAddr);

    await factoryClient.updateGovecAddr({ addr: govecAddr });

    console.log("Factory contract was instantiated at the following address:", factoryAddr);

    const uploadInfo = Object.assign(uploadRes, { govecAddr, factoryAddr });
    writeInCacheFolder("uploadInfo.json", JSON.stringify(uploadInfo, null, 2));
}

main();
