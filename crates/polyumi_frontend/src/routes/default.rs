use actix_web::{ Responder, ResponseError };
use polyumi_models::polyumi::error::{ ResourceKind, ErrorModelKind };

pub async fn default() -> impl Responder {
	ErrorModelKind::NotFound {
		resource_kind: ResourceKind::Route,
		resource_reference: None
	}.model().error_response()
}