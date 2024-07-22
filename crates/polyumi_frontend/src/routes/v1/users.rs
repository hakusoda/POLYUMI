use actix_web::{ delete, get, web, HttpRequest, HttpResponse };
use polyumi_models::{
	hakumi::{
		user::{
			connection::ConnectionModel,
			inbox::InboxItemModel
		},
		GroupModel, UserModel
	},
	polyumi::error::{ ResourceKind, ErrorModelKind }
};
use polyumi_util::{
	id::{
		marker::{ ConnectionMarker, UserMarker },
		Id
	},
	PG_POOL
};
use std::pin::Pin;

use crate::{
	auth::get_session_from_request,
	Result
};

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("user")
		.service(user_get)
		.service(web::scope("{user_id}")
			.service(user_groups)
			.service(user_inbox)
			.service(user_connections)
			.service(web::scope("connection")
				.service(delete_user_connection)
			)
		)
	);
}

#[get("{user_ref}")]
async fn user_get(path: web::Path<String>) -> Result<HttpResponse> {
	match UserModel::get(&path).await? {
		Some(model) => Ok(HttpResponse::Ok().json(model)),
		None => Err(ErrorModelKind::not_found(ResourceKind::User, Some(path)))
	}
}

#[get("groups")]
async fn user_groups(path: web::Path<String>) -> Result<HttpResponse> {
	let user = UserModel::get(&path)
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::User, Some(path)))?;

	let group_ids = UserModel::get_groups(user.id).await?;
	Ok(HttpResponse::Ok().json(GroupModel::get_many(&group_ids).await?))
}

#[get("inbox")]
async fn user_inbox(request: HttpRequest, payload: web::Bytes) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;
	session.verify_request(&request, &payload)?;
	
	Ok(InboxItemModel::get_user_many(session.user_id)
		.await
		.map(|x| HttpResponse::Ok().json(x))?
	)
}

#[get("connections")]
async fn user_connections(request: HttpRequest, path: web::Path<Id<UserMarker>>) -> Result<HttpResponse> {
	let session_user_id = get_session_from_request(&request)
		.await?
		.as_ref()
		.map(|x| x.user_id);
	
	let user_id = *path;
	let models = ConnectionModel::get_user_many(user_id)
		.await?;
	Ok(HttpResponse::Ok().json(if session_user_id == Some(user_id) {
		models
	} else {
		models
			.into_iter()
			.filter(|x| x.is_public)
			.collect()
	}))
}

#[delete("{connection_id}")]
async fn delete_user_connection(request: HttpRequest, payload: web::Bytes, path: web::Path<(Id<UserMarker>, Id<ConnectionMarker>)>) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;
	session.verify_request(&request, &payload)?;

	let (user_id, connection_id) = *path;
	if user_id != session.user_id {
		return Err(ErrorModelKind::MissingPermission.model());
	}

	let connection = ConnectionModel::get(connection_id)
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::UserConnection, Some(connection_id)))?;
	if connection.user_id != user_id {
		return Err(ErrorModelKind::MissingPermission.model());
	}

	sqlx::query!(
		"
		DELETE FROM user_connections
		WHERE id = $1
		",
		connection_id.value
	)
		.execute(&*Pin::static_ref(&PG_POOL).await)
		.await?;
	
	Ok(HttpResponse::Ok().into())
}