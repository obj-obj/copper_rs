//! Fetching data from the Mojang API.

use crate::version::{self, Profile};
use chrono::DateTime;
use reqwest::Error;
use std::{fs::File, path::PathBuf, time::UNIX_EPOCH};
use tracing::{error, info, warn};

/// Gets the version manifest from the Mojang API, or from a cache file as a fallback. If a response is successfully fetched from the API, also saves it to the cache file.
pub async fn get_version_manifest(cache_path: impl ToString) -> version::Manifest {
	let path = &cache_path.to_string();

	match fetch_version_manifest().await {
		Ok(manifest) => {
			if let Ok(cache_file) = File::create(path) {
				if let Err(err) = serde_json::to_writer(cache_file, &manifest) {
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
pub async fn fetch_version_manifest() -> Result<version::Manifest, Error> {
	match reqwest::get("https://launchermeta.mojang.com/mc/game/version_manifest.json").await {
		Ok(res) => {
			return res.json::<version::Manifest>().await;
		}
		Err(err) => Err(err),
	}
}

/// Gets a cached profile, only sending an API request if the local profile is out of date.
pub async fn get_profile(version: &version::Entry, cache_path: &PathBuf) -> Result<Profile, Error> {
	if let Ok(cache_file) = File::open(cache_path) {
		match cache_file.metadata() {
			Ok(metadata) => match metadata.modified() {
				Ok(modify_time) => {
					// If the file was modified after the modify date on the version manifest
					if modify_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
						> DateTime::parse_from_rfc3339(&version.time)
							.unwrap()
							.timestamp_millis() as u128
					{
						match serde_json::from_reader::<File, Profile>(cache_file) {
							Ok(version_profile) => {
								return Ok(version_profile);
							}
							Err(err) => {
								warn!(
									"Error getting {}'s profile from cache!\n\t{err}",
									version.id
								);
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
	info!("{} is invalid, updating...", version.id);
	let version_profile = fetch_profile(version).await;
	let cache_file = File::create(cache_path).unwrap();
	if let Ok(data) = &version_profile {
		if let Err(err) = serde_json::to_writer(cache_file, &data) {
			error!("Error writing version profile to cache!\n\t{err}");
		};
	}
	info!("Updated {}.", version.id);
	return version_profile;
}

/// Returns a specific profile from the Mojang API.
pub async fn fetch_profile(version: &version::Entry) -> Result<Profile, Error> {
	match reqwest::get(&version.url).await {
		Ok(res) => res.json::<Profile>().await,
		Err(err) => Err(err),
	}
}
