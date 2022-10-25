import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { FactoryClient as FactoryC } from "@vectis/types";
import { coin } from "@cosmjs/stargate";

import CWClient from "./cosmwasm";
import * as CHAINS from "../config/chains";

import type { Chains } from "../config/chains";
import type { FactoryT } from "@vectis/types";

class FactoryClient extends FactoryC {
    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, sender, contractAddress);
    }

    static createFactoryInstMsg(
        chainName: Chains,
        proxyCodeId: number,
        multisigCodeId: number,
        govecMinter?: string | null
    ): FactoryT.InstantiateMsg {
        const { addressPrefix, feeToken } = CHAINS[chainName];
        return {
            proxy_code_id: proxyCodeId,
            proxy_multisig_code_id: multisigCodeId,
            addr_prefix: addressPrefix,
            wallet_fee: coin(10000000, feeToken) as FactoryT.Coin,
            govec_minter: govecMinter,
        };
    }

    static async instantiate(
        client: CWClient,
        codeId: number,
        msg: FactoryT.InstantiateMsg,
        initialFunds: FactoryT.Coin[]
    ) {
        const [{ address }] = await client.getAccounts();

        const { contractAddress } = await client.instantiate(
            address,
            codeId,
            msg as unknown as Record<string, string>,
            "Wallet Factory",
            "auto",
            {
                funds: initialFunds,
            }
        );

        return new FactoryClient(client, address, contractAddress);
    }
}

export default FactoryClient;
