use actix_web::{ get, web, HttpRequest, HttpResponse };
use polyumi_models::{
	hakumi::{
		user::inbox::InboxItemModel,
		GroupModel, UserModel
	},
	polyumi::error::{ ResourceKind, ErrorModelKind }
};

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
async fn user_connections(request: HttpRequest, payload: web::Bytes) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;
	session.verify_request(&request, &payload)?;
	
	Ok(InboxItemModel::get_user_many(session.user_id)
		.await
		.map(|x| HttpResponse::Ok().json(x))?
	)
}