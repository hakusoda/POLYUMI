use actix_web::{ get, web, HttpResponse };
use polyumi_models::{
	hakumi::cafe::{ CafeModel, CafeOrderModel },
	polyumi::error::{ ResourceKind, ErrorModelKind }
};

use crate::Result;

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("cafe")
		.service(cafe_get)
		.service(web::scope("{cafe_id}")
			.service(get_cafe_orders)
		)
	);
}

#[get("{cafe_ref}")]
async fn cafe_get(path: web::Path<u64>) -> Result<HttpResponse> {
	match CafeModel::get(*path).await? {
		Some(model) => Ok(HttpResponse::Ok().json(model)),
		None => Err(ErrorModelKind::not_found(ResourceKind::Group, Some(path)))
	}
}

#[get("orders")]
async fn get_cafe_orders(path: web::Path<u64>) -> Result<HttpResponse> {
	let orders = CafeOrderModel::get_cafe_many(*path).await?;
	Ok(HttpResponse::Ok().json(orders))
}