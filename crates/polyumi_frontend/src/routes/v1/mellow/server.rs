use actix_web::{ web, patch, HttpRequest, HttpResponse };
use polyumi_cache::CACHE;
use polyumi_models::{
	hakumi::{
		group::GroupMembershipModel,
		user::connection::ConnectionKind
	},
	mellow::model_event::{ ModelEventKind, ModelKind },
	polyumi::error::ErrorModelKind
};
use polyumi_util::{
	id::{
		marker::{ ConnectionMarker, UserMarker },
		Id
	},
	PG_POOL
};
use serde::{ Deserialize, Serialize };
use std::pin::Pin;
use twilight_model::id::{
	marker::GuildMarker as DiscordGuildMarker,
	Id as DiscordId
};
use validator::Validate;

use crate::{
	auth::get_session_from_request,
	Result
};

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("server")
		.service(web::scope("{server_id}")
			.service(update_syncing_settings)
			.service(update_user_settings)
		)
	);
}

pub async fn verify_membership(server_id: DiscordId<DiscordGuildMarker>, user_id: Id<UserMarker>) -> Result<()> {
	let server = CACHE
		.mellow
		.server(server_id)
		.await
		.map_err(|_| ErrorModelKind::Cache.model())?;
	if
		let Some(owner_group_id) = server.owner_group_id &&
		GroupMembershipModel::get_user(owner_group_id, user_id)
			.await?
			.is_some()
	{
		return Ok(());
	}
	if server.owner_user_id == Some(user_id) {
		return Ok(());
	}

	Err(ErrorModelKind::MissingPermission.model())
}

#[derive(Deserialize, Validate)]
struct UpdateSyncingSettings {
	#[serde(default)]
	allow_forced_syncing: Option<bool>,

	#[serde(default, with = "serde_with::rust::double_option")]
	#[validate(length(max = 32))]
	default_nickname: Option<Option<String>>,

	#[serde(default, with = "serde_with::rust::double_option")]
	skip_onboarding_to: Option<Option<ConnectionKind>>
}

#[patch("syncing/settings")]
async fn update_syncing_settings(request: HttpRequest, path: web::Path<u64>, payload: web::Json<UpdateSyncingSettings>) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;

	let server_id: DiscordId<DiscordGuildMarker> = DiscordId::new_checked(*path)
		.ok_or_else(|| ErrorModelKind::InvalidParams.model())?;
	verify_membership(server_id, session.user_id)
		.await?;

	let pinned = Pin::static_ref(&PG_POOL).await;
	let mut transaction = pinned
		.begin()
		.await?;

		if let Some(allow_forced_syncing) = &payload.allow_forced_syncing {
			sqlx::query!(
				"
				UPDATE mellow_servers
				SET allow_forced_syncing = $2
				WHERE id = $1
				",
				server_id.get() as i64,
				allow_forced_syncing
			)
				.execute(&mut *transaction)
				.await?;
		}

	if let Some(default_nickname) = &payload.default_nickname {
		sqlx::query!(
			"
			UPDATE mellow_servers
			SET default_nickname = $2
			WHERE id = $1
			",
			server_id.get() as i64,
			default_nickname.as_deref()
		)
			.execute(&mut *transaction)
			.await?;
	}

	if let Some(skip_onboarding_to) = &payload.skip_onboarding_to {
		sqlx::query!(
			"
			UPDATE mellow_servers
			SET skip_onboarding_to = $2
			WHERE id = $1
			",
			server_id.get() as i64,
			skip_onboarding_to
				.as_ref()
				.map(|x| x.discriminant() as i16)
		)
			.execute(&mut *transaction)
			.await?;
	}

	transaction
		.commit()
		.await?;

	tokio::spawn(
		ModelEventKind::Updated
			.build(ModelKind::Server(server_id))
			.send()
	);

	Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, Validate)]
struct UpdateUserSettings {
	#[validate(length(max = 16))]
	user_connections: Vec<UpdateUserSettingsUserConnection>
}

#[derive(Deserialize, Serialize)]
struct UpdateUserSettingsUserConnection {
	id: Id<ConnectionMarker>
}

#[patch("member/{user_id}/settings")]
async fn update_user_settings(request: HttpRequest, path: web::Path<(u64, Id<UserMarker>)>, payload: web::Json<UpdateUserSettings>) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;

	let server_id: DiscordId<DiscordGuildMarker> = DiscordId::new_checked(path.0)
		.ok_or_else(|| ErrorModelKind::InvalidParams.model())?;
	let user_id = path.1;
	if user_id != session.user_id {
		return Err(ErrorModelKind::MissingPermission.model());
	}

	// TODO: verify existence of provided user connections
	sqlx::query!(
		"
		INSERT INTO mellow_user_server_settings (server_id, user_connections, user_id)
		VALUES ($1, $2, $3)
		ON CONFLICT (server_id, user_id)
		DO UPDATE SET user_connections = $2
		",
		server_id.get() as i64,
		serde_json::to_value(&payload.user_connections)?,
		user_id.value
	)
		.execute(&*Pin::static_ref(&PG_POOL).await)
		.await?;

	tokio::spawn(
		ModelEventKind::Updated
			.build(ModelKind::UserSettings(server_id, user_id))
			.send()
	);

	Ok(HttpResponse::Ok().finish())
}