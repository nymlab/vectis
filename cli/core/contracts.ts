import { Addr } from "@vectis/types/contracts/FactoryContract";
import { StdFee } from "@cosmjs/amino";
import { SigningCosmWasmClient, UploadResult } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee, defaultUploadFee, walletFee } from "./utils/fee";
import { coin } from "@cosmjs/stargate";
import { InstantiateMsg as FactoryInstantiateMsg } from "@vectis/types/contracts/FactoryContract";
import { InstantiateMsg as GovecInstantiateMsg } from "@vectis/types/contracts/GovecContract";
import { getContract, writeInCacheFolder } from "./utils/fs";

import {
    factoryCodePath,
    adminAddr,
    proxyCodePath,
    fixMultiSigCodePath,
    govecCodePath,
    stakingCodePath,
    daoCodePath,
    voteCodePath,
    proposalSingleCodePath,
    coinMinDenom,
    addrPrefix,
    adminMnemonic,
} from "./utils/constants";
import { createSigningClient } from "./utils/utils";

export async function uploadContract(
    client: SigningCosmWasmClient,
    codePath: string,
    senderAddress?: Addr,
    uploadFee: StdFee | number | "auto" = defaultUploadFee
): Promise<UploadResult> {
    const code = getContract(codePath);
    return client.upload(senderAddress ?? adminAddr, code, uploadFee);
}

export const FACTORY_INITIAL_FUND = coin(10000000, coinMinDenom!);

/**
 * Uploads contracts needed for e2e tests
 *  - factory
 *  - proxy
 *  - cw3 fixed mulitisig
 *  - govec token contract
 *  - dao-contracts: stake_cw20
 *  - dao-contracts: cw20_staked_balance_voting
 *  - dao-contracts: core
 *  - dao-contracts: proposal-single
 *
 *  Current version of DAO contracts: 6831b7f706b16989b3cfac00cab1c2545d1b524 (on mainnet)
 *
 * @param client Signing client
 */
export async function uploadContracts(client: SigningCosmWasmClient): Promise<{
    factoryRes: UploadResult;
    proxyRes: UploadResult;
    multisigRes: UploadResult;
    govecRes: UploadResult;
    stakingRes: UploadResult;
    daoRes: UploadResult;
    voteRes: UploadResult;
    proposalSingleRes: UploadResult;
}> {
    // Upload required contracts
    const factoryRes = await uploadContract(client, factoryCodePath);
    const proxyRes = await uploadContract(client, proxyCodePath);
    const multisigRes = await uploadContract(client, fixMultiSigCodePath);
    const govecRes = await uploadContract(client, govecCodePath);
    const stakingRes = await uploadContract(client, stakingCodePath);
    const daoRes = await uploadContract(client, daoCodePath);
    const voteRes = await uploadContract(client, voteCodePath);
    const proposalSingleRes = await uploadContract(client, proposalSingleCodePath);
    // console.log(factoryRes, proxyRes, multisigRes, govecRes, stakingRes, daoRes, voteRes, proposalSingleRes);

    return {
        factoryRes,
        proxyRes,
        multisigRes,
        govecRes,
        stakingRes,
        daoRes,
        voteRes,
        proposalSingleRes,
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

export async function instantiateGovec(
    client: SigningCosmWasmClient,
    govecCodeId: number,
    minter?: string,
    minterCap?: string
): Promise<{
    govecAddr: Addr;
}> {
    const m = minter ? { minter: minter!, cap: minterCap } : null;
    const instantiate: GovecInstantiateMsg = {
        name: "Govec",
        symbol: "GOVEC",
        initial_balances: [],
        minter: m,
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
