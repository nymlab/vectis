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
            nonce,
        };
    }

    async relayTxFromSelf(cosmosMsgs: CosmosMsgForEmpty[]): Promise<ExecuteResult> {
        let msgs = toCosmosMsg(cosmosMsgs);
        let mock_signature = toCosmosMsg("siganture");
        let relayTxMsg: ProxyT.RelayTransaction = {
            message: msgs,
            signature: mock_signature,
            nonce: 0,
        };
        let proxyMsg: ProxyT.ExecuteMsg = { relay: { transaction: relayTxMsg } };
        let txMsgs = {
            msgs: [
                {
                    wasm: {
                        execute: {
                            contract_addr: this.contractAddress,
                            funds: [],
                            msg: toCosmosMsg(proxyMsg),
                        },
                    },
                },
            ],
        };
        return this.execute(txMsgs);
    }
}
