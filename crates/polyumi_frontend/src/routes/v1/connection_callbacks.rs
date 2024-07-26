use actix_web::{
	cookie::{ time::OffsetDateTime, Cookie, SameSite },
	http::header::LOCATION,
	web,
	HttpRequest, HttpResponse, Responder,
	get
};
use chrono::{ TimeDelta, Utc };
use jsonwebtoken::{ encode, Header };
use polyumi_cache::CACHE;
use polyumi_models::{
	hakumi::user::connection::{ ConnectionKind, ConnectionModel },
	mellow::model_event::{ ModelEventKind, ModelKind },
	polyumi::error::ErrorModelKind
};
use polyumi_util::{
	id::{
		marker::UserMarker,
		Id
	},
	get_json, post_json,
	PG_POOL
};
use serde::{ Deserialize, Serialize };
use std::{
	collections::HashMap,
	pin::Pin
};
use twilight_model::id::{
	marker::UserMarker as DiscordUserMarker,
	Id as DiscordId
};

use crate::{
	auth::{ AUTH_JWT_DURATION, ENCODING_KEY, get_session_from_request },
	Result
};

pub fn config(config: &mut web::ServiceConfig) {
	config.service(connection_callback);
}

#[derive(Deserialize)]
struct CallbackQuery {
	code: Option<String>,
	redirect_uri: Option<String>,
	state: Option<String>
}

#[derive(Deserialize)]
struct BasicToken {
	access_token: String,
	refresh_token: String,
	expires_in: u32,
	token_type: String
}

#[derive(Deserialize)]
struct DiscordUser {
	id: DiscordId<DiscordUserMarker>,
	avatar: Option<String>,
	username: String,
	global_name: Option<String>
}

#[derive(Deserialize)]
#[serde(tag = "data")]
struct PatreonUser {
	data: PatreonUserData
}

#[derive(Deserialize)]
struct PatreonUserData {
	id: String,
	attributes: PatreonUserAttributes
}

#[derive(Deserialize)]
struct PatreonUserAttributes {
	full_name: String,
	image_url: String
}

struct CallbackResponse {
	avatar_url: Option<String>,
	display_name: Option<String>,
	name: Option<String>,
	oauth_authorisation: Option<(BasicToken, Vec<String>)>,
	sub: String,
	website_url: Option<String>
}

const API_URL: &str = env!("API_URL");
const DISCORD_APP_ID: &str = env!("DISCORD_APP_ID");
const DISCORD_APP_SECRET: &str = env!("DISCORD_APP_SECRET");
const PATREON_APP_ID: &str = env!("PATREON_APP_ID");
const PATREON_APP_SECRET: &str = env!("PATREON_APP_SECRET");
const WEBSITE_URL: &str = env!("WEBSITE_URL");

#[derive(Serialize)]
struct JwtClaims {
	is_mellow_session: bool,
	sub: Id<UserMarker>
}

#[get("connection_callback/{connection_kind}")]
async fn connection_callback(request: HttpRequest, path: web::Path<ConnectionKind>, query: web::Query<CallbackQuery>) -> Result<impl Responder> {
	let session = get_session_from_request(&request)
		.await?;

	let connection_kind = path.into_inner();
	let (mellow_server_id, user_id) = if let Some(state) = &query.state && state.starts_with("m1-") {
		if let Some(record) = sqlx::query!(
			"
			DELETE FROM mellow_connection_requests
			WHERE token = $1
			RETURNING server_id, user_id
			",
			&state[3..]
		)
			.fetch_optional(&*Pin::static_ref(&PG_POOL).await)
			.await?
		{
			(Some(DiscordId::new(record.server_id as u64)), Some(record.user_id.into()))
		} else { return Err(ErrorModelKind::InvalidCredentials.model()) }
	} else if !(query.state.as_ref().is_some_and(|x| x.starts_with("mellow_new")) && matches!(connection_kind, ConnectionKind::Discord)) {
		(None, Some(session
			.required()?
			.user_id
		))
	} else { (None, session.as_ref().map(|x| x.user_id)) };

	let response = match &connection_kind {
		ConnectionKind::Discord => {
			let code = query.code
				.clone()
				.ok_or(ErrorModelKind::InvalidQuery)?;

			let params = HashMap::from([
				("code", code),
				("client_id", DISCORD_APP_ID.into()),
				("client_secret", DISCORD_APP_SECRET.into()),
				("grant_type", "authorization_code".into()),
				("redirect_uri", format!("{API_URL}/v1/connection_callback/{}", connection_kind.discriminant()))
			]);
			
			let token: BasicToken = post_json("https://discord.com/api/v10/oauth2/token")
				.form(&params)
				.await?;

			let user: DiscordUser = get_json("https://discord.com/api/v10/users/@me")
				.header("authorization", format!("{} {}", token.token_type, token.access_token))
				.await?;

			let sub = user.id;
			let website_url = Some(format!("https://discord.com/users/{sub}"));
			CallbackResponse {
				avatar_url: user
					.avatar
					.map(|x| format!("https://cdn.discordapp.com/avatars/{sub}/{x}.{}?size=256", if x.starts_with("a_") { "gif" } else { "webp" })),
				display_name: user.global_name,
				name: Some(user.username),
				oauth_authorisation: None,
				sub: sub.to_string(),
				website_url
			}
		},
		ConnectionKind::GitHub => {
			unimplemented!()
		},
		ConnectionKind::Patreon => {
			let code = query.code
				.clone()
				.ok_or(ErrorModelKind::InvalidQuery)?;

			let params = HashMap::from([
				("code", code),
				("client_id", PATREON_APP_ID.into()),
				("client_secret", PATREON_APP_SECRET.into()),
				("grant_type", "authorization_code".into()),
				("redirect_uri", format!("{API_URL}/v1/connection_callback/{}", connection_kind.discriminant()))
			]);
			
			let token: BasicToken = post_json("https://patreon.com/api/oauth2/token")
				.form(&params)
				.await?;

			let user: PatreonUser = get_json("https://www.patreon.com/api/oauth2/v2/identity?fields%5Buser%5D=full_name,image_url")
				.header("authorization", format!("{} {}", token.token_type, token.access_token))
				.await?;

			let sub = user.data.id;
			let website_url = Some(format!("https://www.patreon.com/user?u={sub}"));
			CallbackResponse {
				avatar_url: Some(user.data.attributes.image_url),
				display_name: Some(user.data.attributes.full_name),
				name: Some(sub.clone()),
				oauth_authorisation: Some((token, Vec::new())),
				sub,
				website_url
			}
		},
		ConnectionKind::Roblox |
		ConnectionKind::YouTube => {
			return crate::templates::connection_callback::connection_unsupported(connection_kind, user_id.unwrap())
				.await;
		}
	};

	let pinned = Pin::static_ref(&PG_POOL).await;
	let mut http_response = HttpResponse::Found();
	let user_id = match user_id {
		Some(x) => x,
		None => {
			let sub = sqlx::query!(
				"
				INSERT INTO users (avatar_url, name, username, created_via_mellow)
				VALUES ($1, $2, $3, true)
				RETURNING id
				",
				response.avatar_url,
				response.display_name,
				response.name.as_ref().unwrap_or(&response.sub)
			)
				.fetch_one(&*pinned)
				.await?
				.id
				.into();
			let jwt = encode(&Header::default(), &JwtClaims {
				is_mellow_session: true,
				sub
			}, &ENCODING_KEY)?;
			let cookie = Cookie::build("auth-token", jwt)
				.domain(".hakumi.cafe")
				.expires(OffsetDateTime::now_utc().checked_add(AUTH_JWT_DURATION).unwrap())
				.http_only(false)
				.path("/")
				.same_site(SameSite::None) // haven't researched what this does, just copying it over from the old api
				.finish();
			http_response.cookie(cookie);

			sub
		}
	};
	
	let record = sqlx::query!(
		"
		INSERT INTO user_connections (avatar_url, display_name, sub, type, user_id, username, website_url)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
		RETURNING id
		",
		response.avatar_url,
		response.display_name,
		response.sub,
		connection_kind.discriminant() as i16,
		user_id.value,
		response.name,
		response.website_url
	)
		.fetch_one(&*pinned)
		.await?;

	if let Some((token, scopes)) = response.oauth_authorisation {
		sqlx::query!(
			"
			INSERT INTO user_connection_oauth_authorisations (access_token, refresh_token, token_type, connection_id, expires_at, scopes, user_id)
			VALUES ($1, $2, $3, $4, $5, $6, $7)
			",
			token.access_token,
			token.refresh_token,
			token.token_type,
			record.id,
			Utc::now().checked_add_signed(TimeDelta::seconds(token.expires_in as i64)).unwrap(),
			&scopes,
			user_id.value
		)
			.execute(&*pinned)
			.await?;
	}

	let connection_id = Id::new(record.id);
	let new_model = CACHE
		.hakumi
		.connections
		.entry(connection_id)
		.insert(ConnectionModel {
			id: connection_id,
			sub: response.sub,
			kind: connection_kind.clone(),
			user_id,

			username: response.name,
			display_name: response.display_name,

			avatar_url: response.avatar_url,
			website_url: response.website_url,

			is_public: false,

			oauth_authorisations: Vec::new()
		});
	CACHE
		.hakumi
		.user_connections
		.entry(user_id)
		.or_default()
		.insert(connection_id);

	tokio::spawn(
		ModelEventKind::Created
			.build(ModelKind::UserConnection(user_id, connection_id))
			.send()
	);

	if let Some(server_id) = mellow_server_id {
		sqlx::query!(
			"
			INSERT INTO mellow_user_server_settings (server_id, user_connections, user_id)
			VALUES ($1, $2, $3)
			ON CONFLICT (server_id, user_id)
			DO UPDATE SET user_connections = $2
			",
			server_id.get() as i64,
			serde_json::json!([ { "id": connection_id } ]),
			user_id.value
		)
			.execute(&*pinned)
			.await?;

		tokio::spawn(
			ModelEventKind::Updated
				.build(ModelKind::UserSettings(server_id, user_id))
				.send()
		);

		return crate::templates::connection_callback::mellow_done(connection_kind, server_id, user_id)
			.await;
	}

	let redirect_uri = if let Some(state) = &query.state && state.starts_with("mellow_user_settings") {
		if state.ends_with("as_new_member") {
			let server_id = DiscordId::new_checked(state
				.split('.')
				.nth(1)
				.ok_or_else(|| ErrorModelKind::InvalidQuery.model())?
				.parse()
				.map_err(|_| ErrorModelKind::InvalidQuery.model())?
			).ok_or_else(|| ErrorModelKind::InvalidQuery.model())?;
			sqlx::query!(
				"
				INSERT INTO mellow_user_server_settings (server_id, user_connections, user_id)
				VALUES ($1, $2, $3)
				ON CONFLICT (server_id, user_id)
				DO UPDATE SET user_connections = $2
				",
				server_id.get() as i64,
				serde_json::json!([ { "id": connection_id } ]),
				user_id.value
			)
				.execute(&*pinned)
				.await?;

			tokio::spawn(
				ModelEventKind::Updated
					.build(ModelKind::UserSettings(server_id, user_id))
					.send()
			);
			format!("{WEBSITE_URL}/mellow/server/{}/user_settings?as_new_member", server_id)
		} else {
			format!("{WEBSITE_URL}/mellow/user_settings_popup#{}", urlencoding::encode(&serde_json::to_string(&*new_model)?))
		}
	} else if let Some(state) = &query.state && state.starts_with("mellow_new.") {
		format!("{WEBSITE_URL}/mellow/server/{}/user_settings?as_new_member", &state[11..])
	} else {
		match query.redirect_uri.clone() {
			Some(redirect_uri) => if redirect_uri.starts_with('/') {
				format!("{WEBSITE_URL}{redirect_uri}")
			} else { redirect_uri },
			None => format!("{WEBSITE_URL}/settings/account/connections")
		}
	};
	Ok(http_response
		.append_header((LOCATION, redirect_uri))
		.finish()
	)
}