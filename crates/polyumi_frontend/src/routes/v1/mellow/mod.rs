use actix_web::web;

pub mod server;

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("mellow")
		.configure(server::config)
	);
}