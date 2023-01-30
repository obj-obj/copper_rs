use crate::Result;
use std::{fs::create_dir_all, path::PathBuf};

/// Collection of directories for launcher data.
#[derive(Clone)]
pub struct Paths {
	pub name: String,
	pub cache: PathBuf,
	pub config: PathBuf,

	pub assets: PathBuf,
	pub asset_indexes: PathBuf,
	pub natives: PathBuf,
}
impl Paths {
	/// Generates all sub-directories from a single base directory name.
	pub fn new(name: String) -> Result<Self> {
		let cache = dirs::cache_dir().unwrap().join(&name);
		let config = dirs::config_dir().unwrap().join(&name);

		let assets = cache.join("assets");
		let asset_indexes = assets.join("indexes");
		let natives = cache.join("natives");

		// `create_dir_all` is recursive, which will result in the `cache` and `config` directories also being created.
		create_dir_all(&asset_indexes)?;
		create_dir_all(&natives)?;

		Ok(Self {
			name,
			cache,
			config,

			assets,
			asset_indexes,
			natives,
		})
	}
}
