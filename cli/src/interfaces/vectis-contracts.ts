import type { UploadResult } from "@cosmjs/cosmwasm-stargate";
import { CosmosMsgForEmpty } from "./Cw3Flex.types";
import { Coin } from "@cosmjs/amino";

export interface VectisContractsUploadResult {
    vectis_factory: UploadResult;
    vectis_proxy: UploadResult;
    cw3Fixed: UploadResult;
    vectis_plugin_registry: UploadResult;
    cw3Flex: UploadResult;
    cw4Group: UploadResult;
    vectis_webauthn_authenticator: UploadResult;
}

// These are all the contract
export interface VectisContractsAddrs {
    VectisCommittee: string;
    VectisCommitteeGroup: string;
    Factory: string;
    PluginRegistry: string;
    Webauthn: string;
}

// Proxy WebauthnRelayedTxMsg
//pub struct WebauthnRelayedTxMsg {
//    /// This is the JSON string of the `VectisRelayTx`
//    /// We parse this string in the contract for the correct type
//    /// This is because we need this string to ensure fields are in the
//    /// same order when hashing
//    /// For this authenticator: it is the data to be hashed and becomes the challenge
//    pub signed_data: String,
//    pub auth_data: Binary,
//    pub client_data: Binary,
//}
export type Binary = string;
export interface WebauthnRelayedTxMsg {
    // 'Binary' i.e. toBase64(auth_data: [u8; 37])
    auth_data: Binary;
    // 'Binary' i.e. toBase64(new Uint8Array(response.clientDataJSON)))
    client_data: Binary;
    //  This is the JSON string of the `VectisRelayTx`
    //  VectisRelayTx { message: []CosmosMsg, "nonce": number, "sponsor_fee": Coin }
    //  For this authenticator: it is the data to be hashed and becomes the challenge
    //  e.g. the string is
    //  {"messages":[{"bank":{"send":{"amount":[{"denom":"ujunox","amount":"10"}],"to_address":"juno1qc6cq2lsd0vccceups73auddtu2p6pymwd6ush"}}}],"nonce":0}
    signed_data: string;
}

export interface VectisRelayedTx {
    messages: CosmosMsgForEmpty[];
    nonce: number;
    sponsor_fee?: Coin | null;
}
