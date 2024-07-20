use dashmap::{
	mapref::one::Ref,
	DashMap
};
use polyumi_models::polyumi::{
	auth::{ PasskeyModel, PasskeyChallengeModel },
	SessionModel
};
use polyumi_util::id::{
	marker::PasskeyMarker,
	Id
};

use crate::Result;

#[derive(Default)]
pub struct PolyumiCache {
	pub passkeys: DashMap<String, PasskeyModel>,
	pub passkey_challenges: DashMap<Id<PasskeyMarker>, PasskeyChallengeModel>,
	pub sessions: DashMap<String, SessionModel>
}

impl PolyumiCache {
	pub async fn passkey(&self, passkey_id: &str) -> Result<Ref<'_, String, PasskeyModel>> {
		Ok(match self.passkeys.get(passkey_id) {
			Some(x) => x,
			None => {
				let model = PasskeyModel::get(passkey_id)
					.await?
					.unwrap();
				self.passkeys
					.entry(passkey_id.to_string())
					.insert(model)
					.downgrade()
			}
		})
	}

	pub fn passkey_challenge(&self, challenge_id: Id<PasskeyMarker>) -> Option<Ref<'_, Id<PasskeyMarker>, PasskeyChallengeModel>> {
		self.passkey_challenges.get(&challenge_id)
	}
}