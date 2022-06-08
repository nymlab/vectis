import { Addr } from "@vectis/types/contracts/FactoryContract";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee, walletFee } from "../utils/fee";
import { coin } from "@cosmjs/stargate";
import { InstantiateMsg as FactoryInstantiateMsg } from "@vectis/types/contracts/FactoryContract";
import { addrPrefix, adminAddr, coinMinDenom } from "@vectis/core/utils/constants";

export const FACTORY_INITIAL_FUND = coin(10000000, coinMinDenom);

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
            funds: [FACTORY_INITIAL_FUND],
        }
    );

    return {
        factoryAddr: contractAddress,
    };
}
