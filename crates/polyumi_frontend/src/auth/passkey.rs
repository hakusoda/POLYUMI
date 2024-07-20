use base64::Engine;
use base64urlsafedata::Base64UrlSafeData;
use once_cell::sync::Lazy;
use polyumi_models::polyumi::error::ErrorModelKind;
use serde::{ Deserialize, Serialize };
use std::collections::BTreeMap;
use webauthn_rs_core::{
	crypto::compute_sha256,
	error::WebauthnError,
	internals::AuthenticatorData,
	proto::{
		AuthenticatorAssertionResponseRaw,
		Ceremony,
		COSEKey
	}
};

use crate::Result;

static RP_ID_HASH: Lazy<[u8; 32]> = Lazy::new(|| compute_sha256("hakumi.cafe".as_bytes()));

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationSignedExtensions {
    #[serde(flatten)]
    pub unknown_keys: BTreeMap<String, serde_cbor_2::Value>,
}

pub struct Authentication;
impl Ceremony for Authentication {
    type SignedExtensions = AuthenticationSignedExtensions;
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CollectedClientData {
	#[serde(rename = "type")]
	type_: String,
	//challenge: Base64UrlSafeData,
	challenge: String,
	#[serde(flatten)]
	unknown_keys: BTreeMap<String, serde_json::value::Value>,
}

// https://github.com/kanidm/webauthn-rs/blob/master/webauthn-rs-core/src/internals.rs#L499
struct AuthenticatorAssertionResponse<T: Ceremony> {
    authenticator_data: AuthenticatorData<T>,
    authenticator_data_bytes: Vec<u8>,
    client_data: CollectedClientData,
    client_data_bytes: Vec<u8>,
    signature: Vec<u8>,
    _user_handle: Option<Vec<u8>>
}

impl<T: Ceremony> TryFrom<&AuthenticatorAssertionResponseRaw> for AuthenticatorAssertionResponse<T> {
    type Error = WebauthnError;
    fn try_from(aarr: &AuthenticatorAssertionResponseRaw) -> core::result::Result<Self, Self::Error> {
        Ok(AuthenticatorAssertionResponse {
            authenticator_data: AuthenticatorData::try_from(aarr.authenticator_data.as_ref())?,
            authenticator_data_bytes: aarr.authenticator_data.clone().into(),
            client_data: serde_json::from_slice(aarr.client_data_json.as_ref())
                .map_err(WebauthnError::ParseJSONFailure)?,
            client_data_bytes: aarr.client_data_json.clone().into(),
            signature: aarr.signature.clone().into(),
            _user_handle: aarr.user_handle.clone().map(|uh| uh.into()),
        })
    }
}

pub fn verify_sign_in(challenge: Vec<u8>, public_key: &Base64UrlSafeData, response: &AuthenticatorAssertionResponseRaw) -> Result<()> {
	let data: AuthenticatorAssertionResponse<Authentication> = AuthenticatorAssertionResponse::try_from(response)
		.map_err(|_| ErrorModelKind::InvalidCredentials.model())?;

	let client_data = &data.client_data;
	if client_data.type_ != "webauthn.get" {
		println!("1");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}

	let c = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(&client_data.challenge).unwrap();
	println!("{}", c.len());
	println!("{:?}\n", c.as_slice());
	println!("{:?}", challenge);
	if c.as_slice() != challenge {
		println!("2");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}
	if !data.authenticator_data.user_present {
		println!("3");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}
	if !data.authenticator_data.user_verified {
		println!("4");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}
	if data.authenticator_data_bytes[..32] != *RP_ID_HASH {
		println!("5");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}
	
	let client_data_json_hash = compute_sha256(data.client_data_bytes.as_slice());
	let verification_data: Vec<u8> = data
		.authenticator_data_bytes
		.iter()
		.chain(client_data_json_hash.iter())
		.copied()
		.collect();

	let public_key: serde_cbor_2::Value = serde_cbor_2::from_slice(public_key)
		.map_err(|_| { println!("6"); ErrorModelKind::InvalidCredentials.model() })?;
	let cose_key = COSEKey::try_from(&public_key)
		.map_err(|_| { println!("7"); ErrorModelKind::InvalidCredentials.model() })?;

	let verified = cose_key.verify_signature(&data.signature, &verification_data)
		.map_err(|_| { println!("8"); ErrorModelKind::InvalidCredentials.model() })?;
	if !verified {
		println!("failed verification");
		return Err(ErrorModelKind::InvalidCredentials.model());
	}

	Ok(())
}