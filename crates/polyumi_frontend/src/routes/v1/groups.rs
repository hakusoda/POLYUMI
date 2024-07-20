use std::pin::Pin;
use sqlx::QueryBuilder;
use serde::{ Serialize, Deserialize };
use chrono::{ Utc, DateTime };
use actix_web::{ get, web, post, HttpRequest, HttpResponse };
use polyumi_util::{ id::{ marker::{ GroupMarker, UserMarker }, Id }, PG_POOL };
use polyumi_models::{
	hakumi::{
		group::{ GroupModel, GroupMembershipModel },
		UserModel
	},
	polyumi::error::{ ResourceKind, ErrorModelKind }
};

use crate::{
	auth::get_session_from_request,
	Result
};

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("group")
		.service(group_get)
		.service(web::scope("{group_id}")
			.service(get_group_membership)
			.service(get_group_memberships)
			.service(invite_group_members)
		)
	);
}

#[get("{group_ref}")]
async fn group_get(path: web::Path<String>) -> Result<HttpResponse> {
	match GroupModel::get(&path).await? {
		Some(model) => Ok(HttpResponse::Ok().json(model)),
		None => Err(ErrorModelKind::not_found(ResourceKind::Group, Some(path)))
	}
}

#[derive(Serialize)]
struct GroupMembership {
	created_at: DateTime<Utc>,

	is_invited: bool,
	is_owner: bool,
	is_pending: bool,

	group_id: Id<GroupMarker>,
	user: UserModel
}

#[get("membership")]
async fn get_group_membership(request: HttpRequest, path: web::Path<Id<GroupMarker>>) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;
	let user_id = session.user_id;

	let user = UserModel::get(&user_id.to_string())
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::User, None::<String>))?;
	let membership = GroupMembershipModel::get_user(*path, user_id)
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::GroupMembership, None::<String>))?;
	Ok(HttpResponse::Ok().json(GroupMembership {
		created_at: membership.created_at,
		
		is_invited: membership.is_invited,
		is_owner: membership.is_owner,
		is_pending: membership.is_pending,

		group_id: membership.group_id,
		user
	}))
}

#[get("memberships")]
async fn get_group_memberships(path: web::Path<Id<GroupMarker>>) -> Result<HttpResponse> {
	// TODO: needs to be a paginated response
	let memberships: Vec<_> = GroupMembershipModel::get_group(*path)
		.await?
		.into_iter()
		.filter(|x| !x.is_pending)
		.collect();
	let user_ids: Vec<Id<UserMarker>> = memberships.iter().map(|x| x.user_id).collect();
	let users = UserModel::get_many(&user_ids).await?;
	Ok(HttpResponse::Ok().json(
		memberships
			.into_iter()
			.flat_map(|x|
				users
					.iter()
					.find(|y| y.id == x.user_id)
					.map(|user| GroupMembership {
						created_at: x.created_at,

						is_invited: x.is_invited,
						is_owner: x.is_owner,
						is_pending: x.is_pending,
		
						group_id: x.group_id,
						user: user.clone()
					})
			)
			.collect::<Vec<_>>()
	))
}

#[derive(Deserialize)]
pub struct InviteGroupMembers {
	user_ids: Vec<Id<UserMarker>>
}

#[post("memberships")]
async fn invite_group_members(request: HttpRequest, path: web::Path<Id<GroupMarker>>, payload: web::Json<InviteGroupMembers>) -> Result<HttpResponse> {
	let session = get_session_from_request(&request)
		.await?
		.required()?;
	//session.verify_request(&request, &payload)?;

	let _group = GroupModel::get(&path.to_string())
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::Group, Some(&path)))?;
	let _group_membership = GroupMembershipModel::get_user(*path, session.user_id)
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::User, None::<String>))?;

	let group_memberships = GroupMembershipModel::get_group(*path).await?;
	for user_id in &payload.user_ids {
		if let Some(membership) = group_memberships.iter().find(|x| &x.user_id == user_id) {
			return Err(if membership.is_pending {
				ErrorModelKind::UserAlreadyInGroup { user_id: *user_id }
			} else {
				ErrorModelKind::UserAlreadyPendingInGroup { user_id: *user_id }
			}.model());
		}
	}

	let pinned = Pin::static_ref(&PG_POOL).await;
	let mut query = QueryBuilder::new("INSERT INTO team_members (inviter_id, is_invited, is_pending, team_id, user_id)");
	query.push_values(payload.user_ids.iter(), |mut builder, user_id| {
		builder
			.push_bind(session.user_id.value)
			.push_bind(true)
			.push_bind(true)
			.push_bind(path.value)
			.push_bind(user_id.value);
	});

	query
		.build()
		.execute(pinned.get_ref())
		.await?;

	Ok(HttpResponse::Ok().into())
}