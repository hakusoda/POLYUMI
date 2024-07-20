#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Base64 Decode Error: {0}")]
	Base64Decode(#[from] base64::DecodeError),

	#[error("Base64 Decode Slice Error: {0}")]
	Base64DecodeSlice(#[from] base64::DecodeSliceError),

	#[error("ECDSA Error: {0}")]
	EcdsaError(#[from] p384::ecdsa::Error),

	#[error("Reqwest")]
	Reqwest(#[from] reqwest::Error),

	#[error("Serde JSON")]
	SerdeJson(#[from] serde_json::Error),

	#[error("Sha2 Invalid Length")]
	Sha2InvalidLength(#[from] sha2::digest::InvalidLength),

	#[error("SIMD JSON")]
	SimdJson(#[from] simd_json::Error),

	#[error("SQLx Error: {0}")]
	Sqlx(#[from] sqlx::Error),

	#[error("Missing Signatuer")]
	MissingSignature
}

pub type Result<T> = core::result::Result<T, Error>;