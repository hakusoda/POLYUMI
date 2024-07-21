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

pub mod connection;
pub mod inbox;

#[derive(Clone, Serialize)]
pub struct UserModel {
	pub id: Id<UserMarker>,
	pub bio: Option<String>,
	pub name: Option<String>,
	pub flags: u8,
	pub username: String,
	pub avatar_url: Option<String>,
	pub banner_url: Option<String>,
	pub created_at: DateTime<Utc>,
	pub profile_status: Option<String>,
	pub profile_cafe_id: Option<u64>,
	pub profile_theme_accent_colour: u32,
	pub profile_theme_primary_colour: u32
}

impl UserModel {
	pub fn display_name(&self) -> &str {
		self.name
			.as_ref()
			.unwrap_or(&self.username)
	}

	pub async fn get(user_ref: &str) -> Result<Option<Self>> {
		Self::get_many(&[user_ref])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many<T: Display + Hash + Eq + PartialEq + Clone + Debug>(user_refs: &[T]) -> Result<Vec<Self>> {
		if user_refs.is_empty() {
			return Ok(vec![]);
		}
		
		let pinned = Pin::static_ref(&PG_POOL).await;

		let user_ids: Vec<Uuid> = user_refs
			.iter()
			.flat_map(|x| Uuid::parse_str(&x.to_string()).ok())
			.collect();
		let slugs: Vec<String> = user_refs
			.iter()
			.map(|x| x.to_string().to_lowercase())
			.collect();

		Ok(sqlx::query!(
			r#"
			SELECT u.id id, u.bio bio, u.name name, u.flags flags, u.username username, u.avatar_url avatar_url, u.banner_url banner_url, u.created_at created_at, u.profile_status profile_status, c.id as "profile_cafe_id?", u.theme_accent_colour theme_accent_colour, u.theme_primary_colour theme_primary_colour
			FROM users u
			LEFT JOIN cafes c ON c.owner_user_id = u.id AND c.kind = 'profile'
			WHERE u.id = ANY($1) OR LOWER(u.username) = ANY($2)
			"#,
			&user_ids,
			&slugs
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					id: u.id.into(),
					bio: u.bio,
					name: u.name,
					flags: u.flags as u8,
					username: u.username,
					avatar_url: u.avatar_url,
					banner_url: u.banner_url,
					created_at: u.created_at,
					profile_status: u.profile_status,
					profile_cafe_id: u.profile_cafe_id.map(|x| x as u64),
					profile_theme_accent_colour: u.theme_accent_colour as u32,
					profile_theme_primary_colour: u.theme_primary_colour as u32
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}

	pub async fn get_groups(user_id: Id<UserMarker>) -> Result<Vec<Id<GroupMarker>>> {
		let pinned = Pin::static_ref(&PG_POOL).await;
		Ok(sqlx::query!(
			"
			SELECT g.id FROM teams g
			INNER JOIN team_members gm ON gm.team_id = g.id AND NOT gm.is_pending
			WHERE gm.user_id = $1
			ORDER BY g.display_name DESC, g.name DESC
			",
			user_id.value
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(u.id.into());
				async move { Ok(acc) }
			})
			.await?
		)
	}
}