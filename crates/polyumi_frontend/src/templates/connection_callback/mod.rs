use actix_web::HttpResponse;
use polyumi_cache::CACHE;
use polyumi_models::{
	hakumi::user::{ connection::ConnectionKind, UserModel },
	polyumi::error::ErrorModelKind
};
use polyumi_util::id::{ marker::UserMarker, Id };
use twilight_model::id::{ marker::GuildMarker, Id as DiscordId };

use crate::Result;

pub async fn connection_unsupported(connection_kind: ConnectionKind, user_id: Id<UserMarker>) -> Result<HttpResponse> {
	let user = UserModel::get_many(&[user_id])
		.await?
		.into_iter()
		.next()
		.unwrap();
	
	Ok(HttpResponse::Ok()
		.append_header(("content-type", "text/html; charset=utf-8"))
		.body(
			include_str!("connection_unsupported.html")
				.replace("{{ connection_kind }}", &format!("{connection_kind:?}"))
				.replace("{{ user_name }}", user.display_name())
		)
	)
}

pub async fn mellow_done(connection_kind: ConnectionKind, server_id: DiscordId<GuildMarker>, user_id: Id<UserMarker>) -> Result<HttpResponse> {
	let mut body = include_str!("mellow_done.html")
		.replace("{{ connection_kind }}", &format!("{connection_kind:?}"));

	let server = CACHE
		.mellow
		.server(server_id)
		.await
		.map_err(|_| ErrorModelKind::Cache.model())?;
	body = body
		.replace("{{ server_avatar }}", server.avatar_url.as_deref().unwrap_or(""))
		.replace("{{ server_name }}", &server.name);

	let user = UserModel::get_many(&[user_id])
		.await?
		.into_iter()
		.next()
		.unwrap();
	
	Ok(HttpResponse::Ok()
		.append_header(("content-type", "text/html; charset=utf-8"))
		.body(
			body
				.replace("{{ user_avatar }}", user.avatar_url.as_deref().unwrap_or(""))
				.replace("{{ user_name }}", user.display_name())
		)
	)
}