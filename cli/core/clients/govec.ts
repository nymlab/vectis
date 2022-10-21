import CWClient from "./cosmwasm";
import { GovecClient as GovecC, GovecT } from "@vectis/types";

export const marketingProject = "https://vectis.nymlab.it";
export const marketingDescription = `Govec is the governance token for Vectis DAO. One token is assigned to a Vectis wallet upon creation.`;

interface InstantiateGovec {
    initial_balances: GovecT.Cw20Coin[];
    factory?: string | null;
    minterCap?: string;
    marketing?: GovecT.MarketingInfoResponse;
    dao_tunnel?: string | null;
    staking_addr?: string | null;
}

class GovecClient extends GovecC {
    constructor(client: CWClient, sender: string, contractAddr: string) {
        super(client, sender, contractAddr);
    }

    static async instantiate(client: CWClient, codeId: number, msg: InstantiateGovec) {
        const instantiate: GovecT.InstantiateMsg = {
            name: "Govec",
            symbol: "GOVEC",
            factory: msg.factory,
            initial_balances: msg.initial_balances,
            mint_cap: msg.minterCap,
            marketing: msg.marketing,
            staking_addr: msg.staking_addr,
            dao_tunnel: msg.dao_tunnel,
        };

        const { contractAddress } = await client.instantiate(
            client.sender,
            codeId,
            instantiate as unknown as Record<string, unknown>,
            "Govec",
            "auto",
            {
                admin: client.sender,
            }
        );

        return new GovecClient(client, client.sender, contractAddress);
    }

    static createVectisMarketingInfo(marketingAddr: string) {
        return {
            project: marketingProject,
            description: marketingDescription,
            marketing: marketingAddr,
            logo: null,
        };
    }
}

export default GovecClient;
