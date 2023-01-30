use crate::{
	store::{fetch, fetch_data, force_update},
	structs::{profile::Download, version_manifest, AssetIndex, Profile, VersionManifest},
	Result,
};
use chrono::DateTime;
use std::{fs::File, time::UNIX_EPOCH};

const MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

pub async fn get_profile(version: &version_manifest::Entry) -> Result<Profile> {
	if let Ok(cache_file) = File::open(fetch(None, version.url.clone()).await?.path) {
		let modified_local = cache_file
			.metadata()?
			.modified()?
			.duration_since(UNIX_EPOCH)?
			.as_millis();
		let modified_remote =
			DateTime::parse_from_rfc3339(&version.time)?.timestamp_millis() as u128;

		if modified_local > modified_remote {
			if let Ok(profile) = serde_json::from_reader(cache_file) {
				return Ok(profile);
			}
		}
	}

	// If execution reaches this point, the local file is out of date or invalid.
	match serde_json::from_reader(File::open(force_update(version.url.clone()).await?.path)?) {
		Ok(data) => Ok(data),
		Err(err) => Err(Box::new(err)),
	}
}

pub async fn get_asset_index(index: &Download) -> Result<AssetIndex> {
	fetch_data(None, index.url.clone()).await
}

pub async fn fetch_version_manifest() -> Result<VersionManifest> {
	fetch_data(None, MANIFEST_URL.to_string()).await
}

pub async fn fetch_profile(version: &version_manifest::Entry) -> Result<Profile> {
	fetch_data(None, version.url.clone()).await
}
