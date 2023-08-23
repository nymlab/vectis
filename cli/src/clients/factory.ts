import { FactoryClient as FactoryC } from "../interfaces";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import type { FactoryT } from "../interfaces";
import { walletCreationFee } from "../config/fees";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";

class FactoryClient extends FactoryC {
    constructor(cw: CWClient, sender: string, contractAddress: string) {
        super(cw.client, sender, contractAddress);
    }

    static createFactoryInstMsg(chain: Chain, proxyCodeId: number, webauthnCodeId: number): FactoryT.InstantiateMsg {
        const wallet_fee = walletCreationFee(chain);
        const webauthn_inst_msg = {};
        return {
            msg: {
                proxy_code_id: proxyCodeId,
                wallet_fee: wallet_fee as FactoryT.Coin,
                authenticators: [
                    {
                        code_id: webauthnCodeId,
                        inst_msg: toBase64(toUtf8(JSON.stringify(webauthn_inst_msg))),
                        ty: "webauthn",
                    },
                ],
            },
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

    async createWalletWebAuthn(
        pubkey: Uint8Array,
        initial_data: [string, string][],
        label: string,
        plugins: FactoryT.PluginInstallParams[],
        proxy_initial_funds: FactoryT.Coin[],
        relayers: string[]
    ): Promise<ExecuteResult> {
        let authenticatorProvider: FactoryT.AuthenticatorProvider = "vectis";
        let authenticatorType: FactoryT.AuthenticatorType = "webauthn";
        let authenticator: FactoryT.Authenticator = { provider: authenticatorProvider, ty: authenticatorType };
        let controllingEntity: FactoryT.Entity = { auth: authenticator, data: toBase64(pubkey), nonce: 0 };
        let msg: FactoryT.CreateWalletMsg = {
            //controller: Entity;
            controller: controllingEntity,
            label,
            initial_data,
            proxy_initial_funds,
            relayers,
            plugins,
        };
        return this.createWallet({ createWalletMsg: msg }, "auto", "", proxy_initial_funds);
    }
}

export default FactoryClient;
