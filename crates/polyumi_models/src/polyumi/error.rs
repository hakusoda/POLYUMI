use std::fmt::Display;
use serde::Serialize;
use actix_web::{ http::StatusCode, HttpResponse };
use polyumi_util::id::{ marker::UserMarker, Id };

use crate::Error;

#[derive(Debug, Serialize)]
pub struct ErrorModel {
	pub error: ErrorModelKind
}

impl Display for ErrorModel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("ErrorModel")
	}
}

impl actix_web::ResponseError for ErrorModel {
	fn status_code(&self) -> StatusCode {
		match self.error {
			ErrorModelKind::Cache |
			ErrorModelKind::Database |
			ErrorModelKind::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
			ErrorModelKind::NotFound { .. } => StatusCode::NOT_FOUND,
			ErrorModelKind::InvalidCredentials |
			ErrorModelKind::MissingCredentials => StatusCode::UNAUTHORIZED,
			ErrorModelKind::InvalidSignature |
			ErrorModelKind::InvalidParams |
			ErrorModelKind::InvalidQuery |
			ErrorModelKind::MissingSignature |
			ErrorModelKind::UserAlreadyInGroup { .. } |
			ErrorModelKind::UserAlreadyPendingInGroup { .. } => StatusCode::BAD_REQUEST,
			ErrorModelKind::MissingPermission => StatusCode::FORBIDDEN
		}
	}

	fn error_response(&self) -> HttpResponse {
		HttpResponse::build(self.status_code()).json(self)
	}
}

impl From<ErrorModelKind> for ErrorModel {
	fn from(value: ErrorModelKind) -> Self {
		Self {
			error: value
		}
	}
}

impl From<Error> for ErrorModel {
	fn from(value: Error) -> Self {
		Self {
			error: match value {
				Error::Base64Decode(..) |
				Error::Base64DecodeSlice(..) |
				Error::EcdsaError(..) => ErrorModelKind::InvalidSignature,
				Error::MissingSignature => ErrorModelKind::MissingSignature,
				Error::Reqwest(..) |
				Error::SerdeJson(..) |
				Error::Sha2InvalidLength(..) |
				Error::SimdJson(..) => ErrorModelKind::InternalError,
				Error::Sqlx(..) => ErrorModelKind::Database
			}
		}
	}
}

impl From<jsonwebtoken::errors::Error> for ErrorModel {
	fn from(_value: jsonwebtoken::errors::Error) -> Self {
		ErrorModelKind::InternalError.model()
	}
}

impl From<reqwest::Error> for ErrorModel {
	fn from(value: reqwest::Error) -> Self {
		Error::Reqwest(value).into()
	}
}

impl From<serde_json::Error> for ErrorModel {
	fn from(value: serde_json::Error) -> Self {
		Error::SerdeJson(value).into()
	}
}

impl From<sqlx::Error> for ErrorModel {
	fn from(value: sqlx::Error) -> Self {
		Error::Sqlx(value).into()
	}
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ErrorModelKind {
	Cache,
	Database,
	InternalError,
	InvalidParams,
	InvalidQuery,
	NotFound {
		resource_kind: ResourceKind,
		resource_reference: Option<String>
	},
	InvalidCredentials,
	MissingCredentials,
	InvalidSignature,
	MissingSignature,
	MissingPermission,
	UserAlreadyInGroup {
		user_id: Id<UserMarker>
	},
	UserAlreadyPendingInGroup {
		user_id: Id<UserMarker>
	}
}

impl ErrorModelKind {
	pub fn model(self) -> ErrorModel {
		self.into()
	}

	pub fn not_found(resource_kind: ResourceKind, resource_reference: Option<impl ToString>) -> ErrorModel {
		Self::NotFound {
			resource_kind,
			resource_reference: resource_reference.map(|x| x.to_string())
		}.into()
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
	Group,
	GroupMembership,
	PasskeyChallenge,
	Route,
	User,
	VisualScriptingDocument
}