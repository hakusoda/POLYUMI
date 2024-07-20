use base64urlsafedata::Base64UrlSafeData;
use futures::TryStreamExt;
use polyumi_util::{
	id::{
		marker::UserMarker,
		Id
	},
	PG_POOL
};
use std::pin::Pin;

use crate::Result;

pub struct PasskeyModel {
	pub public_key: Base64UrlSafeData,
	pub user_id: Id<UserMarker>
}

impl PasskeyModel {
	pub async fn get(passkey_id: &str) -> Result<Option<Self>> {
		Self::get_many(&[passkey_id.to_string()])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(passkey_ids: &[String]) -> Result<Vec<Self>> {
		if passkey_ids.is_empty() {
			return Ok(Vec::new());
		}
		
		Ok(sqlx::query!(
			"
			SELECT public_key, user_id
			FROM user_devices
			WHERE id = ANY($1)
			",
			passkey_ids
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					public_key: record.public_key.as_bytes().into(),
					user_id: record.user_id.into()
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}