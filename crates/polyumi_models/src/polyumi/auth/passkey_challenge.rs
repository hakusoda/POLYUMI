use polyumi_util::id::{
	marker::PasskeyMarker,
	Id
};
use rand::Rng;

pub struct PasskeyChallengeModel {
	pub id: Id<PasskeyMarker>,
	pub challenge: Vec<u8>
}

impl Default for PasskeyChallengeModel {
	fn default() -> Self {
		let mut rng = rand::thread_rng();
		Self {
			id: Id::default(),
			challenge: rng.r#gen::<[u8; 32]>().to_vec()
		}
	}
}