use dashmap::DashMap;
use futures::TryStreamExt;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use polyumi_util::{
	id::{
		marker::{ ConnectionMarker, UserMarker },
		Id
	},
	PG_POOL
};
use serde::Serialize;
use serde_repr::{ Deserialize_repr, Serialize_repr };
use std::pin::Pin;

use crate::{
	hakumi::OAuthAuthorisationModel,
	Result
};

#[derive(Serialize)]
pub struct ConnectionModel {
	pub id: Id<ConnectionMarker>,
	pub sub: String,
	pub kind: ConnectionKind,
	pub user_id: Id<UserMarker>,

	pub username: Option<String>,
	pub display_name: Option<String>,

	pub avatar_url: Option<String>,
	pub website_url: Option<String>,

	#[serde(skip)]
	pub oauth_authorisations: Vec<OAuthAuthorisationModel>
}

impl ConnectionModel {
	pub async fn get(connection_id: Id<ConnectionMarker>) -> Result<Option<Self>> {
		Self::get_many(&[connection_id])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(connection_ids: &[Id<ConnectionMarker>]) -> Result<Vec<Self>> {
		let connection_ids: Vec<_> = connection_ids
			.iter()
			.map(|x| x.value)
			.collect();

		let mut transaction = Pin::static_ref(&PG_POOL)
			.await
			.begin()
			.await?;
		let oauth_authorisations = sqlx::query!(
			"
			SELECT id, connection_id, token_type, expires_at, access_token, refresh_token
			FROM user_connection_oauth_authorisations
			WHERE connection_id = ANY($1)
			",
			&connection_ids
		)
			.fetch(&mut *transaction)
			.try_fold(DashMap::<Id<ConnectionMarker>, Vec<OAuthAuthorisationModel>>::new(), |acc, record| {
				acc.entry(record.connection_id.into())
					.or_default()
					.push(OAuthAuthorisationModel {
						id: record.id as u64,
						token_type: record.token_type,
						expires_at: record.expires_at,
						access_token: record.access_token,
						refresh_token: record.refresh_token
					});
				async move { Ok(acc) }
			})
			.await?;

		let connections = sqlx::query!(
			"
			SELECT id, sub, type as kind, username, display_name, avatar_url, website_url, user_id
			FROM user_connections
			WHERE id = ANY($1)
			",
			&connection_ids
		)
			.fetch(&mut *transaction)
			.try_fold(Vec::new(), |mut acc, record| {
				let id: Id<ConnectionMarker> = record.id.into();
				acc.push(Self {
					id,
					sub: record.sub,
					kind: ConnectionKind::from_i16(record.kind).unwrap(),
					user_id: record.user_id.into(),

					username: record.username,
					display_name: record.display_name,

					avatar_url: record.avatar_url,
					website_url: record.website_url,

					oauth_authorisations: oauth_authorisations
						.remove(&id)
						.map(|x| x.1)
						.unwrap_or_default()
				});
				async move { Ok(acc) }
			})
			.await?;

		Ok(connections)
	}

	pub async fn user_discord(user_id: Id<UserMarker>) -> Result<Option<Id<UserMarker>>> {
		Ok(sqlx::query!(
			"
			SELECT user_id
			FROM user_connections
			WHERE sub = $1
			",
			user_id.to_string()
		)
			.fetch_optional(&*Pin::static_ref(&PG_POOL).await)
			.await?
			.map(|x| x.user_id.into())
		)
	}
}

#[derive(Clone, Debug, Deserialize_repr, FromPrimitive, Serialize_repr)]
#[repr(u8)]
pub enum ConnectionKind {
	Discord,
	GitHub,
	Roblox,
	YouTube,
	Patreon
}

impl ConnectionKind {
	pub fn discriminant(&self) -> u8 {
		unsafe { *(self as *const Self as *const u8) }
	}
}