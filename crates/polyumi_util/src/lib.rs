#![feature(const_async_blocks, type_alias_impl_trait)]
use actix_cors::Cors;
use async_once_cell::Lazy;
use sqlx::PgPool;

pub mod fetch;
pub mod id;
pub use fetch::*;

pub type PgPoolFuture = impl Future<Output = PgPool>;

pub static PG_POOL: Lazy<PgPool, PgPoolFuture> = Lazy::new(async {
	PgPool::connect(env!("DATABASE_URL"))
		.await
		.unwrap()
});

pub fn default_cors() -> Cors {
	Cors::default()
		.allow_any_origin()
		.allow_any_header()
		.allow_any_method()
		.supports_credentials()
		.max_age(3600)
}