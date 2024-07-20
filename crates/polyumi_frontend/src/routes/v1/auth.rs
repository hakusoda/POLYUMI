use actix_web::{ web, post, HttpResponse };
use polyumi_cache::CACHE;
use polyumi_models::polyumi::auth::PasskeyChallengeModel;

use crate::Result;

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("auth")
		.service(web::scope("passkeys")
			.service(create_passkey_challenge)
			.service(sign_in_with_passkey)
		)
	);
}

#[post("challenges")]
async fn create_passkey_challenge() -> Result<HttpResponse> {
	let challenge = PasskeyChallengeModel::default();

	let mut response = Vec::with_capacity(48);
	response.extend_from_slice(challenge.id.value.as_bytes());
	response.extend_from_slice(&challenge.challenge);

	CACHE
		.polyumi
		.passkey_challenges
		.insert(challenge.id, PasskeyChallengeModel::default());

	Ok(HttpResponse::Ok().body(response))
}

/*#[derive(Deserialize)]
struct SignInWithPasskey {
	challenge_id: Id<PasskeyMarker>,
	passkey_id: String,
	public_key: Base64UrlSafeData,
	response: AuthenticatorAssertionResponseRaw
}*/

#[post("sign_in")]
async fn sign_in_with_passkey(/*payload: web::Json<SignInWithPasskey>*/) -> Result<HttpResponse> {
	unimplemented!("can't figure out why challenges always mismatch");
	/*let challenge = CACHE
		.polyumi
		.passkey_challenge(payload.challenge_id)
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::PasskeyChallenge, Some(payload.challenge_id)))?;

	let passkey = CACHE
		.polyumi
		.passkey(&payload.passkey_id)
		.await
		.unwrap();

	verify_sign_in(challenge.challenge.clone(), &passkey.public_key.clone(), &payload.response)?;

	Ok(HttpResponse::Ok().finish())*/
}