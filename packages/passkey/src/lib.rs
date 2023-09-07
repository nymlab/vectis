
use passkey::{
    authenticator::{Authenticator, UserValidationMethod},
    client::{Client, WebauthnError},
    types::{ctap2::*, rand::random_vec, crypto::sha256, webauthn::*, Bytes, Passkey},
};

use coset::iana;
use url::Url;


// Example of how to set up, register and authenticate with a `Client`.
let challenge_bytes_from_rp: Bytes = random_vec(32).into();

let parameters_from_rp = PublicKeyCredentialParameters {
    ty: PublicKeyCredentialType::PublicKey,
    alg: iana::Algorithm::ES256,
};
let origin = Url::parse("https://future.1password.com").expect("Should parse");
let user_entity = PublicKeyCredentialUserEntity {
    id: random_vec(32).into(),
    display_name: "Johnny Passkey".into(),
    name: "jpasskey@example.org".into(),
};
// First create an Authenticator for the Client to use.
let my_aaguid = Aaguid::new_empty();
let user_validation_method = MyUserValidationMethod {};
// Create the CredentialStore for the Authenticator.
// Option<Passkey> is the simplest possible implementation of CredentialStore
let store: Option<Passkey> = None;
let my_authenticator = Authenticator::new(my_aaguid, store, user_validation_method);

// Create the Client
// If you are creating credentials, you need to declare the Client as mut
let mut my_client = Client::new(my_authenticator);

// The following values, provided as parameters to this function would usually be
// retrieved from a Relying Party according to the context of the application.
let request = CredentialCreationOptions {
    public_key: PublicKeyCredentialCreationOptions {
        rp: PublicKeyCredentialRpEntity {
            id: None, // Leaving the ID as None means use the effective domain
            name: origin.domain().unwrap().into(),
        },
        user: user_entity,
        challenge: challenge_bytes_from_rp,
        pub_key_cred_params: vec![parameters_from_rp],
        timeout: None,
        exclude_credentials: None,
        authenticator_selection: None,
        attestation: AttestationConveyancePreference::None,
        extensions: None,
    },
};

// Now create the credential.
let my_webauthn_credential = my_client.register(&origin, request).await.unwrap();

// Let's try and authenticate.
// Create a challenge that would usually come from the RP.
let challenge_bytes_from_rp: Bytes = random_vec(32).into();
// Now try and authenticate
let credential_request = CredentialRequestOptions {
    public_key: PublicKeyCredentialRequestOptions {
        challenge: challenge_bytes_from_rp,
        timeout: None,
        rp_id: Some(String::from(origin.domain().unwrap())),
        allow_credentials: None,
        user_verification: UserVerificationRequirement::default(),
        extensions: None,
    },
};

let authenticated_cred = my_client
    .authenticate(&origin, credential_request, None)
    .await
    .unwrap();
