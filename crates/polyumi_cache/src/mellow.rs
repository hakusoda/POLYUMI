use dashmap::{
	mapref::one::Ref,
	DashMap
};
use polyumi_models::mellow::ServerModel;
use twilight_model::id::{
	marker::GuildMarker as DiscordGuildMarker,
	Id as DiscordId
};

use crate::Result;

#[derive(Default)]
pub struct MellowCache {
	servers: DashMap<DiscordId<DiscordGuildMarker>, ServerModel>
}

impl MellowCache {
	pub async fn server(&self, server_id: DiscordId<DiscordGuildMarker>) -> Result<Ref<'_, DiscordId<DiscordGuildMarker>, ServerModel>> {
		Ok(match self.servers.get(&server_id) {
			Some(x) => x,
			None => {
				let new_model = ServerModel::get(server_id)
					.await?
					.unwrap();
				self.servers
					.entry(server_id)
					.insert(new_model)
					.downgrade()
			}
		})
	}
}