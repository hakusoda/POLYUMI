use std::pin::Pin;
use serde::{ Serialize, Deserialize };
use chrono::{ Utc, DateTime };
use futures::TryStreamExt;
use polyumi_util::{ id::{ marker::UserMarker, Id }, PG_POOL };

use crate::Result;

#[derive(Serialize)]
pub struct CafeOrderModel {
	pub id: u64,
	pub cafe_id: u64,
	pub author_id: Option<Id<UserMarker>>,

	#[serde(flatten)]
	pub kind: CafeOrderKind,

	pub created_at: DateTime<Utc>
}

impl CafeOrderModel {
	pub async fn get_cafe_many(cafe_id: u64) -> Result<Vec<Self>> {
		let pinned = Pin::static_ref(&PG_POOL).await;

		Ok(sqlx::query!(
			"
			SELECT id, cafe_id, author_id, kind, payload, created_at
			FROM cafe_orders
			WHERE id = $1
			",
			cafe_id as i64
		)
			.fetch(pinned.get_ref())
			.try_fold(Vec::new(), |mut acc, u| {
				acc.push(Self {
					id: u.id as u64,
					cafe_id: u.cafe_id as u64,
					author_id: u.author_id.map(Into::into),

					// unsure if there's a better way to do this...
					kind: serde_json::from_value(serde_json::json!({
						"kind": u.kind,
						"payload": u.payload
					})).unwrap(),

					created_at: u.created_at
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum CafeOrderKind {
	Message(CafeOrderMessage)
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CafeOrderMessage {
	Basic {
		content: String
	}
}