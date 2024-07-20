#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("SQL Error: {0}")]
	SqlError(#[from] sqlx::Error),

	#[error("Model Error: {0}")]
	ModelError(#[from] polyumi_models::Error)
}

pub type Result<T> = core::result::Result<T, Error>;