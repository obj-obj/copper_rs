use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// List of Minecraft versions along with the latest version.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VersionManifest {
	pub latest: LatestVersions,
	#[serde(with = "array_to_hashmap")]
	pub versions: HashMap<String, Entry>,
}
mod array_to_hashmap {
	use super::Entry;
	use serde::{Deserialize, Deserializer, Serializer};
	use std::collections::HashMap;

	pub fn serialize<S>(map: &HashMap<String, Entry>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.collect_seq(map.values())
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Entry>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let mut map = HashMap::new();
		for entry in Vec::<Entry>::deserialize(deserializer)? {
			map.insert(entry.id.to_string(), entry);
		}
		Ok(map)
	}
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
