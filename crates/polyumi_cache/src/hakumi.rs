use dashmap::{ DashMap, DashSet };
use polyumi_models::hakumi::user::connection::ConnectionModel;
use polyumi_util::id::{
	marker::{ ConnectionMarker, UserMarker },
	Id
};

#[derive(Default)]
pub struct HakumiCache {
	pub connections: DashMap<Id<ConnectionMarker>, ConnectionModel>,
	pub user_connections: DashMap<Id<UserMarker>, DashSet<Id<ConnectionMarker>>>
}