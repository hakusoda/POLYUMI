use actix_web::{
	cookie::{ time::Duration, Cookie },
	HttpRequest
};
use dashmap::mapref::one::Ref;
use jsonwebtoken::{ Algorithm, DecodingKey, EncodingKey, Validation };
use once_cell::sync::Lazy;
use polyumi_util::id::{ marker::UserMarker, Id };
use polyumi_cache::CACHE;
use polyumi_models::polyumi::{ error::ErrorModelKind, SessionModel };
use serde::Deserialize;
use std::ops::Deref;

use crate::Result;

pub mod passkey;

pub const AUTH_JWT_DURATION: Duration = Duration::days(365);
pub const AUTH_JWT_KEY: &[u8] = env!("AUTH_JWT_KEY").as_bytes();
pub static DECODING_KEY: Lazy<DecodingKey> = Lazy::new(|| DecodingKey::from_secret(AUTH_JWT_KEY));
pub static ENCODING_KEY: Lazy<EncodingKey> = Lazy::new(|| EncodingKey::from_secret(AUTH_JWT_KEY));
pub static VALIDATION: Lazy<Validation> = Lazy::new(|| {
	let mut validation = Validation::new(Algorithm::HS256);
	validation.required_spec_claims.clear();
	validation.validate_exp = false;
	validation
});

pub struct SessionOption<'a> {
	inner: Option<Ref<'a, String, SessionModel>>
}

impl<'a> SessionOption<'a> {
	pub fn required(self) -> Result<Ref<'a, String, SessionModel>> {
		self
			.inner
			.ok_or(ErrorModelKind::MissingCredentials.model())
	}
}

impl<'a> Deref for SessionOption<'a> {
	type Target = Option<Ref<'a, String, SessionModel>>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<'a> From<Option<Ref<'a, String, SessionModel>>> for SessionOption<'a> {
	fn from(value: Option<Ref<'a, String, SessionModel>>) -> Self {
		Self { inner: value }
	}
}

pub async fn get_session_from_request(request: &HttpRequest) -> Result<SessionOption<'_>> {
	Ok(if let Some(jwt_token_cookie) = get_authorisation_header(request) {
		let jwt_token = jwt_token_cookie.value();
		Some(match CACHE.polyumi.sessions.get(jwt_token) {
			Some(x) => x,
			None => {
				let session = get_session_from_jwt_token(jwt_token).await?;
				CACHE.polyumi.sessions.entry(jwt_token.to_string())
					.insert(session)
					.downgrade()
			}
		})
		// TODO: other stuff here, like scope checking, fancy stuff....
	} else { None }.into())
}

#[derive(Deserialize)]
struct Claims {
	#[serde(default)]
	device_public_key: Option<String>,
	sub: Id<UserMarker>
}

async fn get_session_from_jwt_token(jwt_token: &str) -> Result<SessionModel> {
	let token = jsonwebtoken::decode::<Claims>(jwt_token, &DECODING_KEY, &VALIDATION)
		.map_err(|x| {
			println!("{x}");
			ErrorModelKind::InvalidCredentials.model()
		})?;

	Ok(SessionModel::new(token.claims.sub, token.claims.device_public_key)?)
}

fn get_authorisation_header(request: &HttpRequest) -> Option<Cookie<'static>> {
	request
		.cookie("auth-token")
}