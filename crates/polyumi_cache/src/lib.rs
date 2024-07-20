use once_cell::sync::Lazy;

pub mod error;
pub mod hakumi;
pub mod mellow;
pub mod polyumi;

use hakumi::HakumiCache;
use mellow::MellowCache;
use polyumi::PolyumiCache;

pub use error::{ Error, Result };

#[derive(Default)]
pub struct Cache {
	pub hakumi: HakumiCache,
	pub mellow: MellowCache,
	pub polyumi: PolyumiCache
}

pub static CACHE: Lazy<Cache> = Lazy::new(Cache::default);