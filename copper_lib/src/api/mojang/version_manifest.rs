use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionManifest {
	pub latest: Latest,
	pub versions: Vec<Version>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Latest {
	pub release: String,
	pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
	pub id: String,
	#[serde(alias = "type")]
	pub version_type: String,
	pub url: String,
	pub time: String,
	#[serde(alias = "releaseTime")]
	pub release_time: String,
}
