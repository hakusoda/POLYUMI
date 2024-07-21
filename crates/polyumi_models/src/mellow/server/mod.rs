use futures::TryStreamExt;
use polyumi_util::{
	id::{
		marker::{ GroupMarker, UserMarker },
		Id
	},
	PG_POOL
};
use std::pin::Pin;
use twilight_model::id::{
	marker::GuildMarker as DiscordGuildMarker,
	Id as DiscordId
};

use crate::Result;

pub struct ServerModel {
	pub name: String,
	pub avatar_url: Option<String>,
	pub owner_group_id: Option<Id<GroupMarker>>,
	pub owner_user_id: Option<Id<UserMarker>>
}

impl ServerModel {
	pub async fn get(server_id: DiscordId<DiscordGuildMarker>) -> Result<Option<Self>> {
		Self::get_many(&[server_id])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(server_ids: &[DiscordId<DiscordGuildMarker>]) -> Result<Vec<Self>> {
		if server_ids.is_empty() {
			return Ok(vec![]);
		}
		
		let server_ids: Vec<i64> = server_ids
			.iter()
			.map(|x| x.get() as i64)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT name, avatar_url, owner_team_id, owner_user_id
			FROM mellow_servers
			WHERE id = ANY($1)
			",
			&server_ids
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					name: record.name,
					avatar_url: record.avatar_url,
					owner_group_id: record.owner_team_id.map(Id::new),
					owner_user_id: record.owner_user_id.map(Id::new)
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}