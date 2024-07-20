use actix_web::web;

pub mod auth;
pub mod cafes;
pub mod connection_callbacks;
pub mod groups;
pub mod mellow;
pub mod users;
pub mod visual_scripting;

pub fn config(config: &mut web::ServiceConfig) {
	config.service(
		web::scope("v1")
			.wrap(polyumi_util::default_cors())
			.configure(auth::config)
			.configure(cafes::config)
			.configure(connection_callbacks::config)
			.configure(groups::config)
			.configure(mellow::config)
			.configure(users::config)
	);
}