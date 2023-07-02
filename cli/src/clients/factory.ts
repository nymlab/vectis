import { FactoryClient as FactoryC } from "../interfaces";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import type { FactoryT } from "../interfaces";
import { walletCreationFee } from "../utils/fees";

class FactoryClient extends FactoryC {
    constructor(cw: CWClient, sender: string, contractAddress: string) {
        super(cw.client, sender, contractAddress);
    }

    static createFactoryInstMsg(chain: Chain, proxyCodeId: number, multisigCodeId: number): FactoryT.InstantiateMsg {
        const { addressPrefix } = chain;
        const wallet_fee = walletCreationFee(chain);
        return {
            proxy_code_id: proxyCodeId,
            proxy_multisig_code_id: multisigCodeId,
            addr_prefix: addressPrefix,
            wallet_fee: wallet_fee as FactoryT.Coin,
        };
    }

    static async instantiate(
        cw: CWClient,
        codeId: number,
        msg: FactoryT.InstantiateMsg,
        initialFunds: FactoryT.Coin[]
    ) {
        const { contractAddress } = await cw.client.instantiate(
            cw.sender,
            codeId,
            msg as unknown as Record<string, string>,
            "Wallet Factory",
            "auto",
            {
                funds: initialFunds,
            }
        );

        return new FactoryClient(cw, cw.sender, contractAddress);
    }
}

export default FactoryClient;
