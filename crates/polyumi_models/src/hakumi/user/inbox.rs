use std::pin::Pin;
use serde::Serialize;
use chrono::{ Utc, DateTime };
use polyumi_util::{
	id::{ marker::UserMarker, Id },
	PG_POOL
};

use crate::Result;
use super::UserModel;

#[derive(Serialize)]
pub struct InboxItemModel {
	id: u64,
	kind: String,
	related_users: Vec<UserModel>,
	created_at: DateTime<Utc>
}

impl InboxItemModel {
	pub async fn get_user_many(user_id: Id<UserMarker>) -> Result<Vec<Self>> {
		let pinned = Pin::static_ref(&PG_POOL).await;
		let records = sqlx::query!(
			"
			SELECT id, kind, related_user_ids, created_at
			FROM user_inbox_items
			WHERE user_id = $1
			",
			user_id.value
		)
			.fetch_all(pinned.get_ref())
			.await?;

		let mut items: Vec<Self> = vec![];
		for record in records {
			let related_users = UserModel::get_many(&record.related_user_ids).await?;
			items.push(Self {
				id: record.id as u64,
				kind: record.kind,
				related_users,
				created_at: record.created_at
			});
		}

		Ok(items)
	}
}