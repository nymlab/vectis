import { Addr } from "@vectis/types/contracts/FactoryContract";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee } from "../utils/fee";
import { InstantiateMsg as GovecInstantiateMsg } from "@vectis/types/contracts/GovecContract";
import { adminAddr } from "../utils/constants";

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
        adminAddr,
        govecCodeId,
        instantiate,
        "Govec",
        defaultInstantiateFee
    );

    return {
        govecAddr: contractAddress,
    };
}
