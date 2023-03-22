import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { FactoryClient as FactoryC } from "../interfaces";

import CWClient from "./cosmwasm";
import * as CHAINS from "../config/chains";

import type { Chains } from "../config/chains";
import type { FactoryT } from "../interfaces";
import { walletInitialFunds, govecClaimFee } from "../utils/fees";

class FactoryClient extends FactoryC {
    constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
        super(client, sender, contractAddress);
    }

    static createFactoryInstMsg(
        chainName: Chains,
        proxyCodeId: number,
        multisigCodeId: number
    ): FactoryT.InstantiateMsg {
        const { addressPrefix } = CHAINS[chainName];
        const wallet_fee = walletInitialFunds(CHAINS[chainName]);
        const claim_fee = govecClaimFee(CHAINS[chainName]);
        return {
            proxy_code_id: proxyCodeId,
            proxy_multisig_code_id: multisigCodeId,
            addr_prefix: addressPrefix,
            wallet_fee: wallet_fee as FactoryT.Coin,
            claim_fee: claim_fee as FactoryT.Coin,
        };
    }

    static async instantiate(
        client: CWClient,
        codeId: number,
        msg: FactoryT.InstantiateMsg,
        initialFunds: FactoryT.Coin[]
    ) {
        const { contractAddress } = await client.instantiate(
            client.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Wallet Factory",
            "auto",
            {
                funds: initialFunds,
            }
        );

        return new FactoryClient(client, client.sender, contractAddress);
    }
}

export default FactoryClient;
