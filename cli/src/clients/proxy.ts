import CWClient from "./cosmwasm";
import { ProxyTypes as ProxyT , ProxyClient as ProxyC} from "./contracts";
import { WebauthnRelayedTxMsg } from "../interfaces";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { longToByteArray, toCosmosMsg } from "../utils/enconding";
import { Secp256k1, sha256 } from "@cosmjs/crypto";
import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";

class ProxyClient extends ProxyC {
    constructor(cw: CWClient, sender: string, contractAddr: string) {
        super(cw.client, sender, contractAddr);
    }

    static async createRelayTransactionCosmos(
        mnemonic: string,
        nonce: number,
        jsonMsg: string
    ): Promise<ProxyT.RelayTransaction> {
        const keypair = await CWClient.mnemonicToKeyPair(mnemonic);
        const messageNonceBytes = new Uint8Array([...toUtf8(jsonMsg), ...longToByteArray(nonce)]);
        const messageHash = sha256(messageNonceBytes);
        const signature = (await Secp256k1.createSignature(messageHash, keypair.privkey)).toFixedLength();
        return {
            message: toBase64(toUtf8(jsonMsg)),
            signature: toBase64(Secp256k1.trimRecoveryByte(signature)),
        };
    }

    async relayTxFromSelf(
        signed_data: string,
        auth_data: string,
        client_data: string,
        signature: string
    ): Promise<ExecuteResult> {
        // See interfaces/vectis-contracts.ts for details
        let webauthnRelayedTxMsg = {
            auth_data,
            client_data,
            signed_data,
        };

        console.log(webauthnRelayedTxMsg);

        let relayTxMsg: ProxyT.RelayTransaction = {
            message: toCosmosMsg(webauthnRelayedTxMsg),
            signature,
        };

        console.log(relayTxMsg);
        console.log(JSON.stringify(relayTxMsg));

        return this.authExec({ transaction: relayTxMsg });
    }
}

export default ProxyClient;
