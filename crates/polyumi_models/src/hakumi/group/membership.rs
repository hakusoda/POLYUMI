use std::pin::Pin;
use uuid::Uuid;
use serde::Serialize;
use chrono::{ Utc, DateTime };
use futures::TryStreamExt;
use polyumi_util::{
	id::{ marker::{ GroupMarker, UserMarker }, Id },
	PG_POOL
};

use crate::Result;

#[derive(Serialize)]
pub struct GroupMembershipModel {
	pub created_at: DateTime<Utc>,

	pub is_invited: bool,
	pub is_owner: bool,
	pub is_pending: bool,

	pub group_id: Id<GroupMarker>,
	pub user_id: Id<UserMarker>
}

impl GroupMembershipModel {
	pub async fn get_group(group_id: Id<GroupMarker>) -> Result<Vec<Self>> {
		Self::get_group_many(&[group_id]).await
	}

	pub async fn get_group_many(group_ids: &[Id<GroupMarker>]) -> Result<Vec<Self>> {
		let pinned = Pin::static_ref(&PG_POOL).await;
		let group_ids: Vec<Uuid> = group_ids
			.iter()
			.map(|x| x.value)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT joined_at, is_invited, is_owner, is_pending, team_id, user_id
			FROM team_members
			WHERE team_id = ANY($1)
			",
			&group_ids
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					created_at: u.joined_at,

					is_invited: u.is_invited,
					is_owner: u.is_owner,
					is_pending: u.is_pending,

					group_id: u.team_id.into(),
					user_id: u.user_id.into()
				});
				async move { Ok(acc) }
			})
			.await?
		)
	}

	pub async fn get_user(group_id: Id<GroupMarker>, user_id: Id<UserMarker>) -> Result<Option<Self>> {
		Self::get_many_user(&[group_id], user_id)
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_user_all(user_id: Id<UserMarker>) -> Result<Vec<Self>> {
		let pinned = Pin::static_ref(&PG_POOL).await;
		Ok(sqlx::query!(
			"
			SELECT joined_at, is_invited, is_owner, is_pending, team_id, user_id
			FROM team_members
			WHERE user_id = $1
			",
			user_id.value
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					created_at: u.joined_at,

					is_invited: u.is_invited,
					is_owner: u.is_owner,
					is_pending: u.is_pending,

					group_id: u.team_id.into(),
					user_id: u.user_id.into()
				});
				async move { Ok(acc) }
			})
			.await?
		)
	}

	pub async fn get_many_user(group_ids: &[Id<GroupMarker>], user_id: Id<UserMarker>) -> Result<Vec<Self>> {
		let pinned = Pin::static_ref(&PG_POOL).await;
		let group_ids: Vec<Uuid> = group_ids
			.iter()
			.map(|x| x.value)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT joined_at, is_invited, is_owner, is_pending, team_id, user_id
			FROM team_members
			WHERE team_id = ANY($1) and user_id = $2
			",
			&group_ids,
			user_id.value
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					created_at: u.joined_at,

					is_invited: u.is_invited,
					is_owner: u.is_owner,
					is_pending: u.is_pending,
					
					group_id: u.team_id.into(),
					user_id: u.user_id.into()
				});
				async move { Ok(acc) }
			})
			.await?
		)
	}
}