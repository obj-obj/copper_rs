use crate::Mod;
use default::default;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
	pub id: String,
	pub name: String,
	#[serde(default)]
	pub mods: Vec<Mod>,
}
impl Config {
	pub fn new(id: impl ToString, name: impl ToString) -> Self {
		Self {
			id: id.to_string(),
			name: name.to_string(),
			..default()
		}
	}
}
