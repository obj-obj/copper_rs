use serde::{Deserialize, Serialize};

/// List of Minecraft versions along with the latest version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
	pub latest: LatestVersions,
	pub versions: Vec<Entry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LatestVersions {
	pub release: String,
	pub snapshot: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
	pub id: String,
	#[serde(alias = "type")]
	pub version_type: String,
	pub url: String,
	pub time: String,
	#[serde(alias = "releaseTime")]
	pub release_time: String,
}
