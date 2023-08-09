import CWClient from "./cosmwasm";
import { ProxyClient as ProxyC } from "../interfaces";
import { ProxyT } from "../interfaces";
import { toBase64, toUtf8 } from "@cosmjs/encoding";
import { CosmosMsgForEmpty } from "../interfaces/Proxy.types";
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

    async relayTxFromSelf(cosmosMsgs: CosmosMsgForEmpty[]): Promise<ExecuteResult> {
        //pub struct VectisRelayedTx {
        //    /// messages to be executed on the entity's behalf
        //    pub messages: Vec<CosmosMsg>,
        //    /// nonce of the entity for relayed tx
        //    pub nonce: Nonce,
        //    /// fee for the relaying party
        //    pub sponsor_fee: Option<Coin>,
        //}
        let vectisRelayedTx = {
            messages: cosmosMsgs,
            nonce: 0,
        };

        let mock_auth_data = new Uint8Array([1, 0, 2, 4]);
        // Expected format for clientData
        //let client_data = Buffer.from(JSON.stringify(clientDataJSON))
        let mock_client_data = new Uint8Array([2, 3, 1, 2]);
        let mock_signature = new Uint8Array([2, 3, 1, 2]);

        let webauthnRelayedTxMsg = {
            signed_data: vectisRelayedTx,
            auth_data: toBase64(mock_auth_data),
            client_data: toBase64(mock_client_data),
        };

        let relayTxMsg: ProxyT.RelayTransaction = {
            message: toCosmosMsg(webauthnRelayedTxMsg),
            signature: toBase64(mock_signature),
        };

        return this.relay({ transaction: relayTxMsg });
    }
}

export default ProxyClient;
