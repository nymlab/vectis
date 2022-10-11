import { FactoryT, GovecT } from "@vectis/types";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { defaultInstantiateFee } from "../utils/fee";
import { adminAddr } from "../utils/constants";

export const marketingProject = "https://vectis.nymlab.it";
export const marketingDescription = `Govec is the governance token for Vectis DAO. One token is assigned to a Vectis wallet upon creation.`;

interface InstantiateGovec {
    client: SigningCosmWasmClient;
    govecCodeId: number;
    admin: string;
    initial_balances: GovecT.Cw20Coin[];
    minters?: string[];
    minterCap?: string;
    marketing?: GovecT.MarketingInfoResponse;
}

export async function instantiateGovec({
    client,
    govecCodeId,
    initial_balances,
    admin,
    minters,
    minterCap,
    marketing,
}: InstantiateGovec): Promise<{
    govecAddr: GovecT.Addr;
}> {
    const minterData = minters ? { minters, cap: minterCap } : null;
    const instantiate: GovecT.InstantiateMsg = {
        name: "Govec",
        symbol: "GOVEC",
        // TODO: give admin initial balance
        initial_balances: initial_balances,
        minter: minterData,
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

export const createVectisMarketingInfo = (marketing?: string): GovecT.MarketingInfoResponse => {
    return {
        project: marketingProject,
        description: marketingDescription,
        marketing,
        logo: null,
    };
};
