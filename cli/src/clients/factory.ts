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

    static createFactoryInstMsg(
        chain: Chain,
        proxyCodeId: number,
        multisigCodeId: number,
        webauthnCodeId: number
    ): FactoryT.InstantiateMsg {
        const { addressPrefix } = chain;
        const wallet_fee = walletCreationFee(chain);
        const webauthn_inst_msg = {};
        return {
            proxy_code_id: proxyCodeId,
            proxy_multisig_code_id: multisigCodeId,
            addr_prefix: addressPrefix,
            wallet_fee: wallet_fee as FactoryT.Coin,
            authenticators: [
                {
                    code_id: webauthnCodeId,
                    inst_msg: toBase64(toUtf8(JSON.stringify(webauthn_inst_msg))),
                    ty: "webauthn",
                },
            ],
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

    async createWalletWebAuthn(pubkey: Uint8Array): Promise<ExecuteResult> {
        let authenticatorProvider: FactoryT.AuthenticatorProvider = "vectis";
        let authenticatorType: FactoryT.AuthenticatorType = "webauthn";
        let authenticator: FactoryT.Authenticator = { provider: authenticatorProvider, ty: authenticatorType };
        let controllingEntity: FactoryT.Entity = { auth: authenticator, data: toBase64(pubkey), nonce: 0 };
        let guardians: FactoryT.Guardians = { addresses: [] };
        let msg: FactoryT.CreateWalletMsg = {
            controller: controllingEntity,
            guardians,
            label: "test-proxy",
            proxy_initial_funds: [{ denom: "ujunox", amount: "100" }],
            relayers: [],
        };
        return this.createWallet({ createWalletMsg: msg }, "auto", "", [{ denom: "ujunox", amount: "100" }]);
    }
}

export default FactoryClient;
