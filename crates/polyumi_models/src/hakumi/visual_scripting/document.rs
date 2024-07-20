use futures::TryStreamExt;
use polyumi_util::{
	id::{
		marker::DocumentMarker,
		Id
	},
	PG_POOL
};
use serde::{ Deserialize, Serialize };
use std::{
	fmt::Display,
	pin::Pin
};
use twilight_model::id::{
	marker::GuildMarker,
	Id as DiscordId
};

use crate::Result;
use super::ElementModel;

pub struct DocumentModel {
	pub id: Id<DocumentMarker>,
	pub name: String,
	pub kind: DocumentKind,
	pub active: bool,
	pub definition: Vec<ElementModel>,
	pub mellow_server_id: Option<DiscordId<GuildMarker>>
}

impl DocumentModel {
	pub async fn get(document_id: Id<DocumentMarker>) -> Result<Option<Self>> {
		Self::get_many(&[document_id])
			.await
			.map(|x| x.into_iter().next())
	}

	pub async fn get_many(document_ids: &[Id<DocumentMarker>]) -> Result<Vec<Self>> {
		let document_ids: Vec<_> = document_ids
			.iter()
			.map(|x| x.value)
			.collect();
		Ok(sqlx::query!(
			"
			SELECT id, name, kind, active, definition, mellow_server_id
			FROM visual_scripting_documents
			WHERE id = ANY($1)
			",
			&document_ids
		)
			.fetch(&*Pin::static_ref(&PG_POOL).await)
			.try_fold(Vec::new(), |mut acc, record| {
				acc.push(Self {
					id: record.id.into(),
					name: record.name,
					kind: serde_json::from_str(&format!("\"{}\"", record.kind)).unwrap(),
					active: record.active,
					definition: serde_json::from_value(record.definition).unwrap(),
					mellow_server_id: record.mellow_server_id.map(|x| DiscordId::new(x as u64))
				});

				async move { Ok(acc) }
			})
			.await?
		)
	}
}


#[derive(Eq, Hash, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DocumentKind {
	#[serde(rename = "mellow.command")]
	MellowCommand,

	#[serde(rename = "mellow.discord_event.member_join")]
	MemberJoinEvent,
	#[serde(rename = "mellow.discord_event.message_create")]
	MessageCreatedEvent,
	#[serde(rename = "mellow.discord_event.member.updated")]
	MemberUpdatedEvent,
	#[serde(rename = "mellow.discord_event.member.completed_onboarding")]
	MemberCompletedOnboardingEvent,

	#[serde(rename = "mellow.event.member.synced")]
	MemberSynced
}

impl Display for DocumentKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// how silly is this? how silly? AHHHHHHHhhhhhh
		let string = simd_json::to_string(self).unwrap();
		let chars = string.chars().skip(1);
		write!(f, "{}", chars.clone().take(chars.count() - 1).collect::<String>())
	}
}