import { instantiateFactoryContract, instantiateGovec, uploadContracts } from "./contracts";
import { addrPrefix, adminMnemonic } from "./utils/constants";
import { writeInCacheFolder } from "./utils/fs";
import { createSigningClient } from "./utils/utils";

export async function uploadAndInst() {
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

    console.log("Factory contract was instantiated at the following address:", factoryAddr);

    const uploadInfo = Object.assign(uploadRes, { govecAddr, factoryAddr });
    writeInCacheFolder("uploadInfo.json", JSON.stringify(uploadInfo, null, 2));
}

uploadAndInst();