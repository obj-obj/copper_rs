use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Mod {
	pub id: String,
	pub version: String,
}
