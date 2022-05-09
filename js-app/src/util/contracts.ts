import { Addr } from "./../../types/FactoryContract";
import { StdFee } from "@cosmjs/amino";
import { SigningCosmWasmClient, UploadResult } from "@cosmjs/cosmwasm-stargate";
import { getContract } from "./utils";
import { defaultInstantiateFee, defaultUploadFee, walletFee } from "./fee";
import { coin } from "@cosmjs/stargate";
import { InstantiateMsg as FactoryInstantiateMsg } from "../../types/FactoryContract";
import { InstantiateMsg as GovecInstantiateMsg } from "../../types/GovecContract";

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
 * Uploads contracts needed for e2e tests
 *  - factory
 *  - proxy
 *  - cw3 fixed mulitisig
 *  - govec token contract
 *  - dao-contracts: cw20 staking
 *
 * @param client Signing client
 */
export async function uploadContracts(client: SigningCosmWasmClient): Promise<{
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

    return {
        factoryRes,
        proxyRes,
        multisigRes,
        govecRes,
        stakingRes,
    };
}

export async function instantiateFactoryContract(
    client: SigningCosmWasmClient,
    factoryCodeId: number,
    proxyCodeId: number,
    multisigCodeId: number
): Promise<{
    factoryAddr: Addr;
}> {
    const instantiate: FactoryInstantiateMsg = {
        proxy_code_id: proxyCodeId,
        proxy_multisig_code_id: multisigCodeId,
        addr_prefix: addrPrefix!,
        wallet_fee: walletFee,
    };
    const { contractAddress } = await client.instantiate(
        adminAddr!,
        factoryCodeId,
        instantiate,
        "Wallet Factory",
        defaultInstantiateFee,
        {
            funds: [FACTORY_INITIAL_FUND],
        }
    );

    return {
        factoryAddr: contractAddress,
    };
}

export async function instantiateGovecWithMinter(
    client: SigningCosmWasmClient,
    govecCodeId: number,
    minter: string,
    minterCap?: string
): Promise<{
    govecAddr: Addr;
}> {
    const instantiate: GovecInstantiateMsg = {
        name: "Govec",
        symbol: "GOVEC",
        initial_balances: [],
        minter: { minter: minter, cap: minterCap },
    };
    const { contractAddress } = await client.instantiate(
        adminAddr!,
        govecCodeId,
        instantiate,
        "Govec",
        defaultInstantiateFee
    );

    return {
        govecAddr: contractAddress,
    };
}
