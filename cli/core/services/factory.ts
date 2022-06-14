import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee } from "../utils/fee";
import { coin, Coin } from "@cosmjs/stargate";
import {
    InstantiateMsg as FactoryInstantiateMsg,
    Coin as FactoryCoin,
    Addr,
} from "@vectis/types/contracts/FactoryContract";
import { addrPrefix, adminAddr, coinMinDenom } from "@vectis/core/utils/constants";
import { walletFee } from "@vectis/core/utils/dao-params";

export const FACTORY_INITIAL_FUND = coin(10000000, coinMinDenom);

export async function instantiateFactoryContract(
    client: SigningCosmWasmClient,
    factoryCodeId: number,
    proxyCodeId: number,
    multisigCodeId: number,
    initialFunds: Coin[]
): Promise<{
    factoryAddr: Addr;
}> {
    const instantiate: FactoryInstantiateMsg = {
        proxy_code_id: proxyCodeId,
        proxy_multisig_code_id: multisigCodeId,
        addr_prefix: addrPrefix,
        wallet_fee: walletFee,
    };
    const { contractAddress } = await client.instantiate(
        adminAddr,
        factoryCodeId,
        instantiate,
        "Wallet Factory",
        defaultInstantiateFee,
        {
            funds: initialFunds,
        }
    );

    return {
        factoryAddr: contractAddress,
    };
}

export function createFactoryInstMsg(
    proxyCodeId: number,
    multisigCodeId: number,
    addrPrefix: string,
    walletFee: FactoryCoin
): FactoryInstantiateMsg {
    return {
        proxy_code_id: proxyCodeId,
        proxy_multisig_code_id: multisigCodeId,
        addr_prefix: addrPrefix,
        wallet_fee: walletFee,
    };
}
