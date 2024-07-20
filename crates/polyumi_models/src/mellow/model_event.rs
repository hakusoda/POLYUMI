use hmac::{ Hmac, Mac };
use polyumi_util::{
	id::{
		marker::{ ConnectionMarker, DocumentMarker, UserMarker },
		Id
	},
	HTTP
};
use serde::Serialize;
use sha2::Sha256;
use twilight_model::id::{
	marker::GuildMarker,
	Id as DiscordId
};

use crate::Result;

type HmacSha256 = Hmac<Sha256>;

const ABSOLUTESOLVER: &[u8] = env!("ABSOLUTESOLVER").as_bytes();

pub async fn send(url: &str, body: Vec<u8>, absolutesolver: &str) -> Result<()> {
	HTTP
		.post(url)
		.body(body)
		.header("absolutesolver", absolutesolver)
		.header("content-type", "application/json")
		.send()
		.await?;
	Ok(())
}

#[derive(Debug, Serialize)]
pub struct ModelEventModel {
	pub actionee_id: Option<Id<UserMarker>>,
	pub kind: ModelEventKind,
	pub model: ModelKind
}

impl ModelEventModel {
	pub async fn send(self) -> Result<()> {
		let mut mac = HmacSha256::new_from_slice(ABSOLUTESOLVER)?;
		let body = serde_json::to_vec(&self)?;
		mac.update(&body);

		let result = mac
			.finalize()
			.into_bytes();
		let encoded = hex::encode(result);
		send("https://mellow-internal-api.hakumi.cafe/internal/model_event", body.clone(), &encoded)
			.await?;
		send("https://local-mellow.hakumi.cafe/internal/model_event", body, &encoded)
			.await?;
		Ok(())
	}
}

#[derive(Debug, Serialize)]
pub enum ModelEventKind {
	Created,
	Updated,
	Deleted
}

impl ModelEventKind {
	pub fn build(self, model_kind: ModelKind) -> ModelEventModel {
		ModelEventModel {
			actionee_id: None,
			kind: self,
			model: model_kind
		}
	}
}

#[derive(Debug, Serialize)]
pub enum ModelKind {
	Server(DiscordId<GuildMarker>),
	UserConnection(Id<UserMarker>, Id<ConnectionMarker>),
	UserSettings(DiscordId<GuildMarker>, Id<UserMarker>),
	VisualScriptingDocument(Option<DiscordId<GuildMarker>>, Id<DocumentMarker>)
}