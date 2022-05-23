use std::{fs::create_dir_all, path::PathBuf};

/// Collection of directories for launcher data.
#[derive(Clone)]
pub struct Directories {
	pub folder_name: String,

	pub cache: PathBuf,
	pub config: PathBuf,

	pub assets: PathBuf,
	pub asset_indexes: PathBuf,
	pub instances: PathBuf,
	pub libraries: PathBuf,
	pub natives: PathBuf,
	pub versions: PathBuf,
}
impl Directories {
	/// Generates directories from a name. Uses a lot of `.unwrap()`s because the launcher can't run anyway when it can't access these directories.
	pub fn new(folder_name: impl ToString) -> Self {
		let folder_name = folder_name.to_string();
		let cache = dirs::cache_dir().unwrap().join(&folder_name);
		let config = dirs::config_dir().unwrap().join(&folder_name);

		let assets = cache.join("assets");
		let asset_indexes = assets.join("indexes");
		let instances = config.join("instances");
		let libraries = cache.join("libraries");
		let natives = cache.join("natives");
		let versions = cache.join("versions");

		// `create_dir_all` is recursive, which will result in the `cache` and `config` directories also being created.
		create_dir_all(&asset_indexes).unwrap();
		create_dir_all(&instances).unwrap();
		create_dir_all(&libraries).unwrap();
		create_dir_all(&natives).unwrap();
		create_dir_all(&versions).unwrap();

		Self {
			folder_name,
			cache,
			config,

			assets,
			asset_indexes,
			instances,
			libraries,
			natives,
			versions,
		}
	}
}
