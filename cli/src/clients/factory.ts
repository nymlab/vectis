import { FactoryClient as FactoryC } from "./contracts/";
import CWClient from "./cosmwasm";
import type { Chain } from "../config/chains";
import { FactoryTypes as FactoryT } from "./contracts/";
import { walletCreationFee } from "../config/fees";
import { Account } from "../config/accounts";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";

class FactoryClient extends FactoryC {
    constructor(cw: CWClient, sender: string, contractAddress: string) {
        super(cw.client, sender, contractAddress);
    }

    static createFactoryInstMsg(chain: Chain, walletCreator: Account, proxyCodeId: number, webauthnCodeId: number): FactoryT.InstantiateMsg {
        const wallet_fee = walletCreationFee(chain);
        const webauthn_inst_msg = {};
        return {
            msg: {
                wallet_creator: walletCreator.address,
                default_proxy_code_id: proxyCodeId,
                supported_proxies: [[proxyCodeId, "v1.0.0-rc2"]],
                wallet_fee: wallet_fee as FactoryT.Coin,
                authenticators: [
                    {
                        code_id: webauthnCodeId,
                        inst_msg: toBase64(toUtf8(JSON.stringify(webauthn_inst_msg))),
                        ty: "webauthn",
                    },
                ],
                supported_chains: [
           //         ["theta-testnet-001", { i_b_c: "connection-697" }],
           //         ["elgafar-1", { i_b_c: "connection-658" }],
           //         ["osmo-test-5", { i_b_c: "connection-671" }],
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
        vid: string,
        plugins: FactoryT.PluginInstallParams[],
        proxy_initial_funds: FactoryT.Coin[],
        relayers: string[]
    ): Promise<ExecuteResult> {
        let authenticatorProvider: FactoryT.AuthenticatorProvider = "vectis";
        let authenticatorType: FactoryT.AuthenticatorType = "webauthn";
        let authenticator: FactoryT.Authenticator = { provider: authenticatorProvider, ty: authenticatorType };
        let controllingEntity: FactoryT.Entity = { auth: authenticator, data: toBase64(pubkey), nonce: 0 };
        let msg: FactoryT.CreateWalletMsg = {
            controller: controllingEntity,
            vid,
            initial_data,
            proxy_initial_funds,
            relayers,
            plugins,
            chains: [
                //[
                //    "theta-testnet-001",
                //    '{"version":"ics27-1","encoding":"proto3","tx_type":"sdk_multi_msg","controller_connection_id":"connection-697","host_connection_id":"connection-2707"}',
                //],
            ],
        };
        return this.createWallet({ createWalletMsg: msg }, "auto", "", proxy_initial_funds);
    }
}

export default FactoryClient;
