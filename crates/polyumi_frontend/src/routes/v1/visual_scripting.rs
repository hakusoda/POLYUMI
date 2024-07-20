use actix_web::{ web, post, HttpRequest, HttpResponse };
use polyumi_models::{
	hakumi::visual_scripting::{ DocumentModel, ElementModel },
	mellow::model_event::{ ModelEventKind, ModelKind },
	polyumi::error::{ ResourceKind, ErrorModelKind }
};
use polyumi_util::{
	id::{
		marker::DocumentMarker,
		Id
	},
	PG_POOL
};
use serde::Deserialize;
use std::pin::Pin;
use validator::Validate;

use crate::{
	auth::get_session_from_request,
	routes::v1::mellow::server::verify_membership,
	Result
};

pub fn config(config: &mut web::ServiceConfig) {
	config.service(web::scope("visual_scripting")
		.service(web::scope("document")
			.service(update_document)
		)
	);
}

#[derive(Deserialize, Validate)]
struct UpdateDocument {
	#[validate(length(max = 16))]
	definition: Vec<ElementModel>
}

#[post("{document_id}")]
async fn update_document(request: HttpRequest, path: web::Path<Id<DocumentMarker>>, payload: web::Json<UpdateDocument>) -> Result<HttpResponse> {
	let user_id = get_session_from_request(&request)
		.await?
		.required()?
		.user_id;

	let document_id = *path;
	let document = DocumentModel::get(document_id)
		.await?
		.ok_or_else(|| ErrorModelKind::not_found(ResourceKind::VisualScriptingDocument, Some(document_id)))?;
	match document.mellow_server_id {
		Some(server_id) => verify_membership(server_id, user_id).await?,
		None => return Err(ErrorModelKind::MissingPermission.model())
	};

	sqlx::query!(
		"
		UPDATE visual_scripting_documents
		SET definition = $2
		WHERE id = $1
		",
		document_id.value,
		serde_json::to_value(&payload.definition)?
	)
		.execute(&*Pin::static_ref(&PG_POOL).await)
		.await?;

	tokio::spawn(
		ModelEventKind::Updated
			.build(ModelKind::VisualScriptingDocument(document.mellow_server_id, document_id))
			.send()
	);

	Ok(HttpResponse::Ok().into())
}