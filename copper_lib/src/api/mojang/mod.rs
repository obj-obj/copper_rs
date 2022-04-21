//! Functions related to fetching data from the Mojang API.

use chrono::DateTime;
use reqwest::Error;
use std::{fs::File, time::UNIX_EPOCH};
use tracing::{error, info, warn};

mod version;
pub use version::Version;
mod version_manifest;
pub use version_manifest::VersionManifest;

/// Gets the version manifest from the Mojang API, or from a cache file as a fallback.
/// If a response is successfully fetched from the API, also saves it to the cache file.
pub async fn get_version_manifest<T>(cache_path: T) -> VersionManifest
where
	T: ToString,
{
	let path = &cache_path.to_string();

	match fetch_version_manifest().await {
		Ok(manifest) => {
			if let Ok(cache_file) = File::create(path) {
				if let Err(err) = serde_json::to_writer_pretty(cache_file, &manifest) {
					error!("Error writing version manifest to cache!\n\t{err}");
				}
				return manifest;
			}
		}
		Err(err) => warn!("Error getting version manifest!\n\t{err}"),
	}

	// Intentionally uses unwrap() because the launcher can't run without a version manifest anyway
	let cache_file = File::open(path).unwrap();
	serde_json::from_reader(cache_file).unwrap()
}

/// Attempts to fetch the version manifest from the Mojang API.
pub async fn fetch_version_manifest() -> Result<VersionManifest, Error> {
	match reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json").await {
		Ok(res) => {
			return res.json::<VersionManifest>().await;
		}
		Err(err) => Err(err),
	}
}

/// Gets a cached version, only sending an API request if the local version is out of date.
pub async fn get_version<T>(
	version: &version_manifest::Version,
	cache_path: T,
) -> Result<Version, Error>
where
	T: ToString,
{
	let path = &cache_path.to_string();

	if let Ok(cache_file) = File::open(path) {
		match cache_file.metadata() {
			Ok(metadata) => match metadata.modified() {
				Ok(modify_time) => {
					// If the file was modified after the modify date on the version manifest
					if modify_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
						> DateTime::parse_from_rfc3339(&version.time)
							.unwrap()
							.timestamp_millis() as u128
					{
						match serde_json::from_reader::<File, Version>(cache_file) {
							Ok(version_profile) => {
								info!("{} is up to date.", version.id);
								return Ok(version_profile);
							}
							Err(err) => {
								warn!("Error getting version profile from cache (it probably doesn't exist)!\n\t{err}");
							}
						};
					}
				}
				Err(err) => {
					error!("Error getting modify date for file!\n\t{err}");
				}
			},
			Err(err) => {
				error!("Error getting metadata for file!\n\t{err}");
			}
		}
	}

	// If execution reaches this point, the file is out of date
	info!("{} is out of date, updating...", version.id);
	let version_profile = fetch_version(version).await;
	let cache_file = File::create(path).unwrap();
	if let Ok(data) = &version_profile {
		if let Err(err) = serde_json::to_writer_pretty(cache_file, &data) {
			error!("Error writing version profile to cache!\n\t{err}");
		};
	}
	return version_profile;
}

/// Returns a specific version profile from the Mojang API.
pub async fn fetch_version(version: &version_manifest::Version) -> Result<Version, Error> {
	match reqwest::get(&version.url).await {
		Ok(res) => res.json::<Version>().await,
		Err(err) => Err(err),
	}
}
