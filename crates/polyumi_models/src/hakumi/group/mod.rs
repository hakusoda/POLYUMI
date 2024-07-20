use std::{
	fmt::{ Debug, Display },
	hash::Hash,
	pin::Pin
};
use uuid::Uuid;
use serde::Serialize;
use chrono::{ Utc, DateTime };
use futures::TryStreamExt;
use polyumi_util::{
	id::{ marker::{ GroupMarker, UserMarker }, Id },
	PG_POOL
};

use crate::Result;

pub mod membership;
pub use membership::GroupMembershipModel;

#[derive(Serialize)]
pub struct GroupModel {
	pub id: Id<GroupMarker>,
	pub created_at: DateTime<Utc>,
	pub creator_id: Option<Id<UserMarker>>,

	pub bio: Option<String>,
	pub name: String,
	pub display_name: Option<String>,

	pub avatar_url: Option<String>,
	pub banner_url: Option<String>,
	pub profile_theme_accent_colour: u32,
	pub profile_theme_primary_colour: u32
}

impl GroupModel {
	pub async fn get(group_ref: &str) -> Result<Option<Self>> {
		Self::get_many(&[group_ref])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many<T: Display + Hash + Eq + PartialEq + Clone + Debug>(group_refs: &[T]) -> Result<Vec<Self>> {
		if group_refs.is_empty() {
			return Ok(vec![]);
		}
		
		let pinned = Pin::static_ref(&PG_POOL).await;

		let group_ids: Vec<Uuid> = group_refs
			.iter()
			.flat_map(|x| Uuid::parse_str(&x.to_string()).ok())
			.collect();
		let slugs: Vec<String> = group_refs
			.iter()
			.map(|x| x.to_string().to_lowercase())
			.collect();

		Ok(sqlx::query!(
			"
			SELECT id, created_at, creator_id, bio, name, display_name, avatar_url, banner_url, profile_theme_accent_colour, profile_theme_primary_colour
			FROM teams
			WHERE id = ANY($1) OR LOWER(name) = ANY($2)
			",
			&group_ids,
			&slugs
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					id: u.id.into(),
					created_at: u.created_at,
					creator_id: u.creator_id.map(Into::into),

					bio: u.bio,
					name: u.name,
					display_name: u.display_name,

					avatar_url: u.avatar_url,
					banner_url: u.banner_url,
					profile_theme_accent_colour: u.profile_theme_accent_colour as u32,
					profile_theme_primary_colour: u.profile_theme_primary_colour as u32
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}