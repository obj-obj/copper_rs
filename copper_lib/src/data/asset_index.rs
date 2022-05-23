use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssetIndex {
	pub objects: HashMap<String, Entry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
	pub hash: String,
	pub size: i32,
}
