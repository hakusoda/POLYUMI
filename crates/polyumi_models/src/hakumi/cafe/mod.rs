use std::pin::Pin;
use serde::Serialize;
use chrono::{ Utc, DateTime };
use futures::TryStreamExt;
use polyumi_util::{
	id::{ marker::{ GroupMarker, UserMarker }, Id },
	PG_POOL
};

use crate::Result;

pub mod order;
pub use order::CafeOrderModel;

#[derive(Serialize)]
pub struct CafeModel {
	pub id: u64,
	pub creator_user_id: Option<Id<UserMarker>>,
	pub owner_group_id: Option<Id<GroupMarker>>,
	pub owner_user_id: Option<Id<UserMarker>>,

	pub kind: CafeKind,

	pub created_at: DateTime<Utc>
}

impl CafeModel {
	pub async fn get(cafe_ref: u64) -> Result<Option<Self>> {
		Self::get_many(&[cafe_ref])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(cafe_refs: &[u64]) -> Result<Vec<Self>> {
		if cafe_refs.is_empty() {
			return Ok(vec![]);
		}
		
		let pinned = Pin::static_ref(&PG_POOL).await;

		let cafe_ids: Vec<_> = cafe_refs
			.iter()
			.map(|x| *x as i64)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT id, creator_user_id, owner_group_id, owner_user_id, kind, created_at
			FROM cafes
			WHERE id = ANY($1)
			",
			&cafe_ids
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					id: u.id as u64,
					creator_user_id: u.creator_user_id.map(Into::into),
					owner_group_id: u.owner_group_id.map(Into::into),
					owner_user_id: u.owner_user_id.map(Into::into),

					kind: u.kind.as_str().into(),

					created_at: u.created_at
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CafeKind {
	Unknown,
	Profile
}

impl From<&str> for CafeKind {
	fn from(value: &str) -> Self {
		match value {
			"profile" => Self::Profile,
			_ => Self::Unknown
		}
	}
}