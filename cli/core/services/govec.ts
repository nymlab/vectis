import { Addr } from "@vectis/types/contracts/FactoryContract";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee } from "../utils/fee";
import {
    InstantiateMsg as GovecInstantiateMsg,
    Cw20Coin,
    MarketingInfoResponse,
} from "@vectis/types/contracts/GovecContract";
import { adminAddr } from "../utils/constants";

export const marketingProject = "https://vectis.nymlab.it";
export const marketingDescription = `Govec is the governance token for Vectis DAO. One token is assigned to a Vectis wallet upon creation.`;

interface InstantiateGovec {
    client: SigningCosmWasmClient;
    govecCodeId: number;
    admin: string;
    initial_balances: Cw20Coin[];
    minter?: string;
    minterCap?: string;
    marketing?: MarketingInfoResponse;
}

export async function instantiateGovec({
    client,
    govecCodeId,
    initial_balances,
    admin,
    minter,
    minterCap,
    marketing,
}: InstantiateGovec): Promise<{
    govecAddr: Addr;
}> {
    const m = minter ? { minter: minter!, cap: minterCap } : null;
    const instantiate: GovecInstantiateMsg = {
        name: "Govec",
        symbol: "GOVEC",
        // TODO: give admin initial balance
        initial_balances: initial_balances,
        minter: m,
        marketing,
    };
    const { contractAddress } = await client.instantiate(
        adminAddr,
        govecCodeId,
        instantiate,
        "Govec",
        defaultInstantiateFee,
        {
            admin: admin,
        }
    );

    return {
        govecAddr: contractAddress,
    };
}

export const createVectisMarketingInfo = (marketing?: string): MarketingInfoResponse => {
    return {
        project: marketingProject,
        description: marketingDescription,
        marketing,
        logo: null,
    };
};
