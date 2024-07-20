#![feature(duration_constructors, let_chains)]
use log::info;
use std::pin::Pin;
use actix_web::{
	error::InternalError,
	middleware::Logger,
	web,
	App, HttpResponse, HttpServer
};
use once_cell::sync::Lazy;
use polyumi_util::PG_POOL;
use polyumi_cache::CACHE;
use polyumi_models::polyumi::ErrorModel;

pub mod auth;
pub mod routes;

pub type Result<T> = core::result::Result<T, ErrorModel>;

pub async fn setup_frontend() -> std::io::Result<()> {
	let bind_addr = std::env::var("BIND_ADDRESS")
		.expect("BIND_ADDRESS is not defined");
	info!("starting polyumi_frontend on {bind_addr}");
	
	Lazy::force(&CACHE);
	Lazy::force(&auth::DECODING_KEY);
	Lazy::force(&auth::VALIDATION);
	Pin::static_ref(&PG_POOL).await;

	HttpServer::new(|| {
        App::new()
			.wrap(Logger::new("%r  →  %s, %b bytes, took %Dms"))
            .configure(routes::v1::config)
			.app_data(web::JsonConfig::default().error_handler(|error,_| InternalError::from_response(
				"",
				HttpResponse::BadRequest()
					.content_type("application/json")
					.body(format!(r#"{{"error":"json error: {error}"}}"#)),
			).into()))
			.default_service(web::get().wrap(polyumi_util::default_cors()).to(routes::default::default))
    })
		.bind(bind_addr)?
		.run()
		.await
}