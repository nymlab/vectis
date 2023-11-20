use base64ct::Encoding;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;

pub fn must_create_credential(vid: &str) -> Vec<u8> {
    let output = Command::new("./passkey-cli")
        .arg("create-credential")
        .arg("--vid")
        .arg(vid)
        .output()
        .unwrap();

    if !output.status.success() {
        println!("output: {:?}", output);
        panic!("output failed")
    }

    let output_str = String::from_utf8(output.stdout.clone()).unwrap();
    let vec: Vec<u8> = serde_json_wasm::from_str(&output_str).unwrap();

    vec
}

pub fn must_get_credential(vid: &str, challenge: String) -> AuthenticatorAssertionResponse {
    let output = Command::new("./passkey-cli")
        .arg("get-credential")
        .arg("--vid")
        .arg(vid)
        .arg("--challenge")
        .arg(challenge)
        .output()
        .unwrap();

    if !output.status.success() {
        panic!("output failed")
    }

    let s = String::from_utf8(output.stdout.clone()).unwrap();

    let response: AuthenticatorAssertionResponse = serde_json::from_str(&s).unwrap();

    response
}

pub(crate) fn hash_to_hex_string<'a>(data: &[u8]) -> String {
    let s = HexBinary::from(Sha256::digest(data).to_vec());
    s.to_hex()
}

pub(crate) fn hash_to_base64url_string<'a>(data: &[u8]) -> String {
    base64ct::Base64UrlUnpadded::encode_string(Sha256::digest(data).as_slice())
}

pub(crate) fn de_client_data(data: &[u8]) -> CollectedClientData {
    serde_json_wasm::from_slice(data).unwrap()
}

// ==================================
// Types from passkey
// ==================================

#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Bytes(Vec<u8>);

/// This type represents an authenticator's response to a client’s request for generation of a new
/// authentication assertion given the Relying Party's [challenge](PublicKeyCredentialRequestOptions)
/// and OPTIONAL list of credentials it is aware of. This response contains a cryptographic signature
/// proving possession of the credential private key, and optionally evidence of user consent to a
/// specific transaction.
///
/// <https://w3c.github.io/webauthn/#iface-authenticatorassertionresponse>
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAssertionResponse {
    /// This attribute contains the JSON serialization of [`CollectedClientData`] passed to the
    /// authenticator by the client in order to generate this credential. The exact JSON serialization
    /// MUST be preserved, as the hash of the serialized client data has been computed over it.
    #[serde(rename = "clientDataJSON")]
    pub client_data_json: Vec<u8>,

    /// This attribute contains the authenticator data returned by the authenticator. See [`AuthenticatorData`].
    pub authenticator_data: Vec<u8>,

    /// This attribute contains the raw signature returned from the authenticator.
    pub signature: Vec<u8>,

    /// This attribute contains the user handle returned from the authenticator, or null if the
    /// authenticator did not return a user handle.
    ///
    /// This mirrors the [`PublicKeyCredentialUserEntity::id`] field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_handle: Option<Vec<u8>>,
}
// Source from https://docs.rs/passkey-types/latest/src/passkey_types/webauthn/attestation.rs.html#473-497
//
// We cannot directly import from the crate with some issues with `getrandom` to wasm.
#[cw_serde]
#[serde(rename_all = "camelCase")]
pub struct CollectedClientData {
    /// This member contains the value [`ClientDataType::Create`] when creating new credentials, and
    /// [`ClientDataType::Get`] when getting an assertion from an existing credential. The purpose
    /// of this member is to prevent certain types of signature confusion attacks (where an attacker
    ///  substitutes one legitimate signature for another).
    #[serde(rename = "type")]
    pub ty: ClientDataType,

    /// This member contains the base64url encoding of the challenge provided by the Relying Party.
    /// See the [Cryptographic Challenges] security consideration.
    ///
    /// [Cryptographic Challenges]: https://w3c.github.io/webauthn/#sctn-cryptographic-challenges
    pub challenge: String,

    /// This member contains the fully qualified origin of the requester, as provided to the
    /// authenticator by the client, in the syntax defined by [RFC6454].
    ///
    /// [RFC6454]: https://www.rfc-editor.org/rfc/rfc6454
    pub origin: String,

    /// This OPTIONAL member contains the inverse of the sameOriginWithAncestors argument value that
    /// was passed into the internal method
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cross_origin: Option<bool>,
}

/// Used to limit the values of [`CollectedClientData::ty`] and serializes to static strings.
#[cw_serde]
pub enum ClientDataType {
    /// Serializes to the string `"webauthn.create"`
    #[serde(rename = "webauthn.create")]
    Create,

    /// Serializes to the string `"webauthn.get"`
    #[serde(rename = "webauthn.get")]
    Get,
}
