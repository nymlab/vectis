import { Addr } from "./../../types/FactoryContract";
import { StdFee } from "@cosmjs/amino";
import { SigningCosmWasmClient, UploadResult } from "@cosmjs/cosmwasm-stargate";
import { getContract } from "./utils";
import { defaultInstantiateFee, defaultUploadFee, walletFee } from "./fee";
import { coin } from "@cosmjs/stargate";

import {
    factoryCodePath,
    adminAddr,
    proxyCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    stakingCodePath,
    coinMinDenom,
    addrPrefix,
} from "./env";

export async function uploadContract(
    client: SigningCosmWasmClient,
    codePath: string,
    senderAddress?: Addr,
    uploadFee?: StdFee | number | "auto"
): Promise<UploadResult> {
    const code = getContract(codePath);
    return client.upload(senderAddress ?? adminAddr!, code, uploadFee ?? defaultUploadFee);
}

export const FACTORY_INITIAL_FUND = coin(10000000, coinMinDenom!);

/**
 * Deploys Factory contract
 *
 * @param client Signing client
 */
export async function deployFactoryContract(client: SigningCosmWasmClient): Promise<{
    contractAddress: Addr;
    factoryRes: UploadResult;
    proxyRes: UploadResult;
    multisigRes: UploadResult;
    govecRes: UploadResult;
    stakingRes: UploadResult;
}> {
    // Upload required contracts
    const factoryRes = await uploadContract(client, factoryCodePath!);
    const proxyRes = await uploadContract(client, proxyCodePath!);
    const multisigRes = await uploadContract(client, fixMultiSigCodePath!);
    const govecRes = await uploadContract(client, govecCodePath!);
    const stakingRes = await uploadContract(client, stakingCodePath!);

    const { contractAddress } = await client.instantiate(
        adminAddr!,
        factoryRes.codeId,
        {
            proxy_code_id: proxyRes.codeId,
            proxy_multisig_code_id: multisigRes.codeId,
            govec_code_id: govecRes.codeId,
            staking_code_id: stakingRes.codeId,
            addr_prefix: addrPrefix!,
            wallet_fee: walletFee,
        },
        "Wallet Factory",
        defaultInstantiateFee,
        {
            funds: [FACTORY_INITIAL_FUND],
        }
    );

    return {
        contractAddress,
        factoryRes,
        proxyRes,
        multisigRes,
        govecRes,
        stakingRes,
    };
}
