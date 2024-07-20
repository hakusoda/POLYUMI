use actix_web::HttpRequest;
use base64::prelude::*;
use p384::ecdsa::{ signature::Verifier, Signature, VerifyingKey };
use polyumi_util::id::{ marker::UserMarker, Id };

use crate::{ Error, Result };

pub struct SessionModel {
	pub user_id: Id<UserMarker>,
	pub public_key: Option<VerifyingKey>
}

impl SessionModel {
	pub fn new(user_id: Id<UserMarker>, public_key: Option<String>) -> Result<Self> {
		Ok(Self {
			user_id,
			public_key: if let Some(public_key) = public_key {
				let decoded_key = BASE64_STANDARD.decode(public_key)?;
				Some(VerifyingKey::from_sec1_bytes(&decoded_key)?)
			} else { None }
		})
	}

	pub fn verify_request(&self, request: &HttpRequest, body: &[u8]) -> Result<()> {
		if let Some(public_key) = self.public_key {
			let headers = request.headers();
			let raw_signature = headers.get("haku-sig")
				.ok_or(Error::MissingSignature)?;

			let decoded_signature = BASE64_STANDARD.decode(raw_signature)?;
			let signature = Signature::from_slice(&decoded_signature)?;

			let data = format!("{} {};{}", request.method(), request.path(), BASE64_STANDARD.encode(body));
			public_key.verify(data.as_bytes(), &signature)?;
		}

		Ok(())
	}
}